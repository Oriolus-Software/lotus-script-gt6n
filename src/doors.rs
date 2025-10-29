use lotus_rt_extra::{
    backbone::VehicleBackbone,
    backbone_types,
    doors::{
        DoorControlMode, DoorControlProperties, DoorControlState,
        DoorWarningOutsideRelayWithStopOnSpeedProperties, ElectricSlidingPlugDoorPairPositionState,
        ElectricSlidingPlugDoorPairProperties, ElectricSlidingPlugDoorPairState,
        ElectricSlidingPlugDoorPairTarget, door_control,
        door_warning_outside_relay_with_stop_on_speed,
    },
    observer::{Observer, ObserverVec, changed},
    timers::BlinkRelayProperties,
};

use crate::backbone_special_types::{
    Door1Force, DoorRequest, DoorsAllClosed, DoorsReleased, OverrideNoWarning,
};

const PLUG_RADIUS: f32 = 0.06;
const SHIFT_DISTANCE: f32 = 0.58;
const FRICTION: f32 = 0.05;
const OPEN_END_SPEED: f32 = 0.3;
const OPEN_START_END_CHANGE_POSITION: f32 = 0.6;
const CLOSE_END_SPEED: f32 = 0.1;
const CLOSE_START_END_CHANGE_POSITION: f32 = 0.2;
const TRACTION_STIFTNESS: f32 = 4.0;

#[derive(Clone)]
pub struct DoorsState {
    pub doors_with_controller: Vec<DoorsWithController>,
    pub released: Observer<bool>,
    pub requests: Vec<Observer<bool>>,
    pub door_1_override: Observer<DoorControlMode>,
    pub override_no_warning: Observer<bool>,
    pub all_closed: Observer<bool>,
}

#[derive(Clone)]
pub struct DoorsWithController {
    pub door: ElectricSlidingPlugDoorPairState,
    pub control: DoorControlState,
    pub closed: Observer<bool>,
}

pub fn add_doors(backbone: &mut VehicleBackbone) {
    let ready = backbone.get(backbone_types::SystemsReady).unwrap();
    let released = backbone.create_observer(DoorsReleased);
    let door_1_force = backbone.create_observer(Door1Force);

    let mut requests = vec![];

    for i in 0..4 {
        requests.push(backbone.create_observer(DoorRequest::DoorRight(i as i8)));
    }

    let door_with_controller =
        |door_number: usize,
         start_speed: f32,
         close_start_speed: f32,
         reflection_open: f32,
         reflection_close: f32,
         door_1_force: Option<Observer<DoorControlMode>>| {
            let mut door_target = Observer::<ElectricSlidingPlugDoorPairTarget>::default();

            let door_prop = ElectricSlidingPlugDoorPairProperties::builder()
                .plug_radius(PLUG_RADIUS)
                .shift_distance(SHIFT_DISTANCE)
                .friction(FRICTION)
                .open_start_speed(start_speed)
                .open_end_speed(OPEN_END_SPEED)
                .open_start_end_change_position(OPEN_START_END_CHANGE_POSITION)
                .close_start_speed(close_start_speed)
                .close_end_speed(CLOSE_END_SPEED)
                .close_start_end_change_position(CLOSE_START_END_CHANGE_POSITION)
                .traction_stiftness(TRACTION_STIFTNESS)
                .reflection_open(reflection_open)
                .reflection_close(reflection_close)
                .sound_open_start(format!("Snd_Door_{}_Open_Start", door_number + 1))
                .sound_open_end(format!("Snd_Door_{}_Open_End", door_number + 1))
                .sound_close_start(format!("Snd_Door_{}_Close_Start", door_number + 1))
                .sound_close_transition(format!("Snd_Door_{}_Close_Trans", door_number + 1))
                .sound_close_end(format!("Snd_Door_{}_Close_End", door_number + 1))
                .variable_x_rail(format!("Door_{}_R", door_number + 1))
                .variable_y_blade_a(format!("Door_{}_1", door_number + 1))
                .variable_y_blade_b(format!("Door_{}_2", door_number + 1))
                .build();

            let mut door = door_target.electric_sliding_plug_door_pair(door_prop);

            let control_properties = DoorControlProperties::builder()
                .request_time(6.0)
                .warning_time(2.0)
                .set_system_active(ready.clone())
                .set_request(requests[door_number].clone())
                .set_released(released.clone())
                .set_door_closed(door.position.clone());

            let mut control_properties = if let Some(force) = door_1_force {
                control_properties.set_force(force.clone()).build()
            } else {
                control_properties.build()
            };

            let mut control = door_control(&mut control_properties);

            control.door_target.write_to(&door_target);

            let closed = door
                .position
                .map(|v| *v == ElectricSlidingPlugDoorPairPositionState::FullyClosed);

            DoorsWithController {
                door,
                control,
                closed,
            }
        };

    let doors_with_controller = vec![
        door_with_controller(0, 0.6, 0.5, 0.03, 0.05, Some(door_1_force.clone())),
        door_with_controller(1, 0.65, 0.45, 0.05, 0.05, None),
        door_with_controller(2, 0.62, 0.42, 0.05, 0.05, None),
        door_with_controller(3, 0.58, 0.48, 0.03, 0.05, None),
    ];

    let mut state = DoorsState {
        doors_with_controller: doors_with_controller.clone(),
        released,
        requests,
        door_1_override: door_1_force,
        override_no_warning: backbone.create_observer(OverrideNoWarning),
        all_closed: ObserverVec::<bool>::new(
            doors_with_controller
                .clone()
                .iter()
                .map(|v| v.closed.clone())
                .collect::<Vec<_>>(),
        )
        .all(|v| v),
    };

    backbone.insert(DoorsAllClosed, state.all_closed.clone());

    state
        .door_1_override
        .map(|v| *v == DoorControlMode::Automatic)
        .and_observer(
            &mut state.doors_with_controller[0].control.warning,
            false,
            false,
        )
        .blink_relay(
            BlinkRelayProperties::builder()
                .interval(0.777)
                .on_time(0.388)
                .build(),
        )
        .filter(changed())
        .trigger_sound("Snd_Door_1_Warning")
        .to_float()
        .var_writer("Door_1_WarnlightI");

    let warnings_1_3: Vec<_> = state.doors_with_controller[1..]
        .iter()
        .map(|v| v.control.warning.clone())
        .collect();

    let mut outside_warning_relais = ObserverVec::<bool>::new(warnings_1_3.clone())
        .any(|v| v)
        .and_observer(
            &mut state
                .door_1_override
                .map(|v| *v == DoorControlMode::Automatic),
            false,
            false,
        )
        .blink_relay(
            BlinkRelayProperties::builder()
                .interval(0.777)
                .on_time(0.388)
                .build(),
        );

    warnings_1_3.iter().enumerate().for_each(|(i, v)| {
        v.clone()
            .and_observer(&mut outside_warning_relais, false, false)
            .filter(changed())
            .trigger_sound(format!("Snd_Door_{}_Warning", i + 1))
            .to_float()
            .var_writer(format!("Door_{}_WarnlightI", i + 1));
    });

    let all_doors_closed = ObserverVec::<bool>::new(
        state
            .doors_with_controller
            .iter()
            .map(|v| v.closed.clone())
            .collect::<Vec<_>>(),
    )
    .all(|v| v);

    if let Some(vehicle_speed) = backbone.get(backbone_types::VehicleSpeed) {
        let mut warning_outside_relay = door_warning_outside_relay_with_stop_on_speed(
            DoorWarningOutsideRelayWithStopOnSpeedProperties::builder()
                .timer_after_closed(30.0)
                .max_speed(3.0 / 3.6)
                .released(state.released.clone())
                .all_doors_closed(all_doors_closed.clone())
                .speed(vehicle_speed.clone())
                .build(),
        );

        warning_outside_relay.var_writer("Snd_Relais_Doorwarn");

        let mut outside_warning_blinker_relais = warning_outside_relay.blink_relay(
            BlinkRelayProperties::builder()
                .interval(0.393)
                .on_time(0.196)
                .build(),
        );

        outside_warning_blinker_relais
            .to_float()
            .var_writer("Door_1_WarnlightO")
            .var_writer("Door_234_WarnlightO");
    }
}

#[derive(Clone, Debug)]
pub enum DoorSwitchState {
    Closed,
    Released,
    Open,
    DoorOne,
}
