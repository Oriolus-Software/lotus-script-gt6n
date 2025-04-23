use lotus_script::log;

use lotus_rt_extra::{
    combined::{
        add_blink_relais_multiple_entries, add_blink_relais_with_light_and_sound,
        BlinkRelaisMultipleEntriesProperties, BlinkRelaisWithLightAndSoundProperties,
        LightAndSoundVarPair,
    },
    doors::{
        add_door_control, add_door_warning_outside_relay_with_stop_on_speed,
        add_electric_sliding_plug_door_pair, DoorControlMode, DoorControlProperties,
        DoorControlState, DoorWarningOutsideRelayWithStopOnSpeedProperties,
        ElectricSlidingPlugDoorPairPositionState, ElectricSlidingPlugDoorPairProperties,
        ElectricSlidingPlugDoorPairState,
    },
    simple::{
        add_and, add_blink_relais, add_bool_to_float_var_unit, add_bool_to_sound_unit,
        add_converter, BlinkRelaisProperties, BoolToFloatVarUnitProperties,
        BoolToSoundUnitProperties,
    },
    standard_elements::Shared,
};

const PLUG_RADIUS: f32 = 0.06;
const SHIFT_DISTANCE: f32 = 0.58;
const FRICTION: f32 = 0.05;
const OPEN_END_SPEED: f32 = 0.3;
const OPEN_START_END_CHANGE_POSITION: f32 = 0.6;
const CLOSE_END_SPEED: f32 = 0.1;
const CLOSE_START_END_CHANGE_POSITION: f32 = 0.2;
const TRACTION_STIFTNESS: f32 = 4.0;

#[derive(Clone, Debug)]
pub struct DoorsState {
    pub doors_with_controller: Vec<DoorsWithController>,
    pub released: Shared<bool>,
    pub requests: Vec<Shared<bool>>,
    pub vehicle_speed: Shared<f32>,
    pub door_1_override: Shared<DoorControlMode>,
    pub override_no_warning: Shared<bool>,
    pub all_closed: Shared<bool>,
}

#[derive(Clone, Debug)]
pub struct DoorsWithController {
    pub door: ElectricSlidingPlugDoorPairState,
    pub control: DoorControlState,
    pub closed: Shared<bool>,
}

pub fn add_doors() -> DoorsState {
    log::info!("Adding doors ------------------------------");

    let system_active = Shared::new(true);

    let released = Shared::new(false);
    let door_1_force = Shared::new(DoorControlMode::default());

    let requests: Vec<_> = std::iter::repeat_with(|| Shared::new(false))
        .take(4)
        .collect();

    let add_door_with_controller =
        |door_number: usize,
         start_speed: f32,
         close_start_speed: f32,
         reflection_open: f32,
         reflection_close: f32,
         door_1_force: Option<Shared<DoorControlMode>>| {
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

            let door = add_electric_sliding_plug_door_pair(door_prop);

            let control_properties = DoorControlProperties::builder()
                .request_time(6.0)
                .warning_time(2.0)
                .set_system_active(system_active.clone())
                .set_request(requests[door_number].clone())
                .set_released(released.clone())
                .get_door_target(door.set_target.clone())
                .set_door_closed(door.position.clone());

            let control_properties = if let Some(force) = door_1_force {
                control_properties.set_force(force.clone()).build()
            } else {
                control_properties.build()
            };

            let control = add_door_control(control_properties);

            let closed = add_converter(
                door.position.clone(),
                |v| *v == ElectricSlidingPlugDoorPairPositionState::FullyClosed,
                None,
            );

            DoorsWithController {
                door,
                control,
                closed,
            }
        };

    let doors_with_controller = vec![
        add_door_with_controller(0, 0.6, 0.5, 0.03, 0.05, Some(door_1_force.clone())),
        add_door_with_controller(1, 0.65, 0.45, 0.05, 0.05, None),
        add_door_with_controller(2, 0.62, 0.42, 0.05, 0.05, None),
        add_door_with_controller(3, 0.58, 0.48, 0.03, 0.05, None),
    ];

    let state = DoorsState {
        doors_with_controller: doors_with_controller.clone(),
        vehicle_speed: Shared::new(0.0),
        released,
        requests,
        door_1_override: door_1_force,
        override_no_warning: Shared::new(false),
        all_closed: add_and(
            doors_with_controller
                .clone()
                .iter()
                .map(|v| v.closed.clone())
                .collect(),
            None,
        ),
    };

    let bla = add_converter(
        state.door_1_override.clone(),
        |v| *v == DoorControlMode::Automatic,
        None,
    );

    add_blink_relais_with_light_and_sound(BlinkRelaisWithLightAndSoundProperties {
        blink_relais_properties: BlinkRelaisProperties {
            interval: 0.777,
            on_time: 0.388,
            reset_time: None,
            set_running: add_and(
                vec![
                    state.doors_with_controller[0].control.warning.clone(),
                    bla.clone(),
                ],
                None,
            ),
        },
        light_and_sound: LightAndSoundVarPair {
            light: "Door_1_WarnlightI".to_string(),
            sound: "Snd_Door_1_Warning".to_string(),
        },
    });

    add_blink_relais_multiple_entries(BlinkRelaisMultipleEntriesProperties {
        interval: 0.777,
        on_time: 0.388,
        reset_time: None,
        entries: vec![
            (
                state.doors_with_controller[1].control.warning.clone(),
                LightAndSoundVarPair {
                    light: "Door_2_WarnlightI".to_string(),
                    sound: "Snd_Door_2_Warning".to_string(),
                },
            ),
            (
                state.doors_with_controller[2].control.warning.clone(),
                LightAndSoundVarPair {
                    light: "Door_3_WarnlightI".to_string(),
                    sound: "Snd_Door_3_Warning".to_string(),
                },
            ),
            (
                state.doors_with_controller[3].control.warning.clone(),
                LightAndSoundVarPair {
                    light: "Door_4_WarnlightI".to_string(),
                    sound: "Snd_Door_4_Warning".to_string(),
                },
            ),
        ],
    });

    let all_doors_closed = add_and(
        state
            .doors_with_controller
            .iter()
            .map(|v| v.closed.clone())
            .collect(),
        None,
    );

    let warning_outside_relay = add_door_warning_outside_relay_with_stop_on_speed(
        DoorWarningOutsideRelayWithStopOnSpeedProperties::builder()
            .timer_after_closed(30.0)
            .max_speed(3.0 / 3.6)
            .released(state.released.clone())
            .all_doors_closed(all_doors_closed.clone())
            .speed(state.vehicle_speed.clone())
            .build(),
    );

    add_bool_to_sound_unit(BoolToSoundUnitProperties {
        set_bool: warning_outside_relay.warning.clone(),
        sound: "Snd_Relais_Doorwarn".to_string(),
    });

    let outside_warning_blinker_relais = add_blink_relais(
        BlinkRelaisProperties {
            interval: 0.393,
            on_time: 0.196,
            reset_time: None,
            set_running: warning_outside_relay.warning.clone(),
        },
        None,
    );

    add_bool_to_float_var_unit(BoolToFloatVarUnitProperties {
        set_bool: outside_warning_blinker_relais.on.clone(),
        float: "Door_1_WarnlightO".to_string(),
    });

    add_bool_to_float_var_unit(BoolToFloatVarUnitProperties {
        set_bool: outside_warning_blinker_relais.on.clone(),
        float: "Door_234_WarnlightO".to_string(),
    });

    state
}

#[derive(Clone, Debug)]
pub enum DoorSwitchState {
    Closed,
    Released,
    Open,
    DoorOne,
}
