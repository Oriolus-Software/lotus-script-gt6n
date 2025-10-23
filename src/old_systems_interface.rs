use lotus_rt::{spawn, wait};
use lotus_rt_extra::{
    cockpit_special::{TokenProperties, TokenSlot, token},
    doors::DoorControlMode,
    input::InputEvent,
    physic::{InertionSliderBumpProperties, InertionSliderProperties, InertionSliderState},
    shared::{Shared, multiple_on_change},
    vehicle_systems::BlinkerState,
};
use lotus_script::var::set_var;

use crate::{
    cockpit::{Cockpit, CockpitRear},
    cockpit_types::{
        BackDriveSwitch, BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState,
    },
    doors::DoorsState,
    lights::LightState,
    misc::MiscState,
    passenger_elements::PassengerElementsState,
    traction::{TractionDirection, TractionState},
};

#[derive(Debug, Clone)]
pub struct SystemStates {
    pub cockpit: Cockpit,
    pub cockpit_rear: CockpitRear,
    pub passenger: PassengerElementsState,
    pub traction: TractionState,
    pub lights: LightState,
    pub misc: MiscState,
    pub doors: DoorsState,
}

#[derive(Debug, Clone, Default)]
struct InterfaceState {
    cockpit_a_active: Shared<bool>,
    cockpit_a_drive: Shared<bool>,
    cockpit_b: Shared<bool>,
    active: Shared<bool>,
    drive: Shared<bool>,
}

#[derive(Clone)]
pub struct Interface {
    systems: SystemStates,
    state: InterfaceState,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Schluessel {
    Vorne,
    Hinten,
}

impl Default for Interface {
    fn default() -> Self {
        let voltage_r = Shared::<f32>::new(1.0);

        let sys = systems_interface(SystemStates {
            cockpit: crate::cockpit::add_cockpit(voltage_r.clone()),
            cockpit_rear: crate::cockpit::add_cockpit_rear(voltage_r.clone()),
            passenger: crate::passenger_elements::passenger_elements(),
            traction: crate::traction::add_traction(),
            lights: crate::lights::add_lights(),
            misc: crate::misc::add_misc(),
            doors: crate::doors::doors(),
        });

        set_var("Coupling_A_vis", true);
        set_var("Coupling_B_vis", true);

        sys
    }
}

pub fn systems_interface(channels: SystemStates) -> Interface {
    let channels_clone = channels.clone();

    let cockpit_a_active = channels_clone
        .cockpit
        .richtungswender
        .process(|r| !matches!(r, RichtungswenderState::O));
    let cockpit_a_drive = channels_clone
        .cockpit
        .richtungswender
        .process(|r| matches!(r, RichtungswenderState::V | RichtungswenderState::R));
    let cockpit_b = channels_clone.cockpit_rear.schloss.clone();

    token::<Schluessel>(
        TokenProperties::builder()
            .standard_position(Schluessel::Vorne)
            .slots(vec![
                TokenSlot::builder()
                    .token(Schluessel::Vorne)
                    .visibility_var("Schluessel_A_RW")
                    .input_event_set(InputEvent::new("InsertKey_Reverser", 0))
                    .input_event_reset(InputEvent::new("Key_Reverser_R", 0))
                    .sound_set("Snd_CP_A_KeyIn")
                    .sound_reset("Snd_CP_A_KeyOut")
                    .locked_deactivate(channels_clone.cockpit.schloss.delay_relay(0.0, 0.1).clone())
                    .build(),
                TokenSlot::builder()
                    .token(Schluessel::Hinten)
                    .visibility_var("Schluessel_H")
                    .input_event_set(InputEvent::new("InsertKey_Reverser", 1))
                    .input_event_reset(InputEvent::new("Key_Reverser_R", 1))
                    .sound_set("Snd_CP_B_KeyIn")
                    .sound_reset("Snd_CP_B_KeyOut")
                    .locked_deactivate(
                        channels_clone
                            .cockpit_rear
                            .schloss
                            .delay_relay(0.0, 0.1)
                            .clone(),
                    )
                    .build(),
            ])
            .build(),
    );

    let interface = Interface {
        systems: channels,
        state: InterfaceState {
            cockpit_a_active: cockpit_a_active.clone(),
            cockpit_a_drive: cockpit_a_drive.clone(),
            cockpit_b: cockpit_b.clone(),
            active: cockpit_a_active.or(&cockpit_b.clone()),
            drive: cockpit_a_drive.or(&cockpit_b.clone()),
        },
    };

    interface.systems.lights.voltage.set(1.0);

    traction_control(&interface);

    outside_lights(&interface);

    blinker_lights(&interface);

    inside_lights(&interface);

    // Cockpit ---------------------------------------

    // interface
    //     .systems
    //     .cockpit
    //     // .klingel
    //     .mg_bremse
    //     .or(&interface.systems.cockpit.klingel)
    //     .and(&interface.state.cockpit_a_active)
    //     .forward(&interface.systems.misc.klingel);

    // interface
    //     .systems
    //     .cockpit
    //     .lightcheck
    //     .and(&interface.state.cockpit_a_active)
    //     .forward(&interface.systems.cockpit.lm_check);

    interface
        .systems
        .traction
        .federspeicher
        .and(&interface.state.cockpit_a_active)
        .forward(&interface.systems.cockpit.lm_federspeicher);

    // Doors ---------------------------------------

    spawn(door_control(
        interface.systems.doors.clone(),
        interface.systems.cockpit.clone(),
        interface.systems.passenger.clone(),
        interface.systems.traction.clone(),
    ));

    // Misc Systems ---------------------------------------

    interface
        .state
        .active
        .loop_sound("Snd_Cabin_IdleI".to_string());

    interface
        .state
        .drive
        .loop_sound("Snd_Cabin_IdleVR".to_string());

    // Z Position ---------------------------------------

    z_position();

    //-----------------------------

    interface
}

fn traction_control(interface: &Interface) {
    let active = interface.state.active.clone();
    let cockpit_b = interface.state.cockpit_b.clone();

    let direction = interface.systems.traction.direction.clone();
    let richtungswender = interface.systems.cockpit.richtungswender.clone();

    multiple_on_change(&[&active.clone(), &richtungswender.clone()], move || {
        direction.set(if active.get() {
            match richtungswender.get() {
                RichtungswenderState::V => TractionDirection::Forward,
                RichtungswenderState::R => TractionDirection::Backward,
                _ => {
                    if cockpit_b.get() {
                        TractionDirection::Backward
                    } else {
                        TractionDirection::Neutral
                    }
                }
            }
        } else {
            TractionDirection::Neutral
        });
    });

    let drive = interface.state.drive.clone();
    let cockpit_a_drive = interface.state.cockpit_a_drive.clone();
    let cockpit_b = interface.state.cockpit_b.clone();

    let sollwertgeber = interface.systems.cockpit.sollwertgeber.clone();
    let rear_fahrschalter = interface.systems.cockpit_rear.fahrschalter.clone();

    let traction_target = interface.systems.traction.target.clone();

    multiple_on_change(
        &[
            &drive.clone(),
            &sollwertgeber.clone(),
            &rear_fahrschalter.clone(),
        ],
        move || {
            traction_target.set(if cockpit_a_drive.get() {
                if sollwertgeber.get() < 0.0 {
                    sollwertgeber.get() * 1.111
                } else {
                    sollwertgeber.get()
                }
            } else if cockpit_b.get() {
                match rear_fahrschalter.get() {
                    BackDriveSwitch::Drive => 0.5,
                    BackDriveSwitch::Neutral => 0.0,
                    BackDriveSwitch::Brake => -0.6,
                    BackDriveSwitch::MaxBrake => -1.0,
                }
            } else {
                0.0
            });
        },
    );

    let cockpit_a_drive = interface.state.cockpit_a_drive.clone();
    let cockpit_mg_bremse = interface.systems.cockpit.mg_bremse.clone();

    let mg_target = interface.systems.traction.mg.clone();

    multiple_on_change(
        &[&cockpit_a_drive.clone(), &cockpit_mg_bremse.clone()],
        move || {
            mg_target.set(cockpit_a_drive.get() && cockpit_mg_bremse.get());
        },
    );

    interface
        .systems
        .cockpit
        .sanden
        .and(&interface.state.cockpit_a_active)
        .forward(&interface.systems.traction.sanding);

    interface
        .systems
        .cockpit
        .federspeicher_overwrite
        .process(|v| v.is_in())
        .and(&interface.state.cockpit_a_active)
        .or(&interface.state.drive.invert())
        .delay_relay(0.3, 0.3)
        .forward(&interface.systems.traction.federspeicher);
}

fn outside_lights(interface: &Interface) {
    // interface
    //     .systems
    //     .cockpit
    //     .beleuchtung_aussen
    //     .process(|sw| *sw != OutsideLightSwitch::Off)
    //     .forward(&interface.systems.lights.stand)
    //     .forward(&interface.systems.lights.rueck)
    //     .forward(&interface.systems.lights.instrumente);

    // interface
    //     .systems
    //     .cockpit
    //     .beleuchtung_aussen
    //     .process(|sw| (*sw == OutsideLightSwitch::Abblend) || (*sw == OutsideLightSwitch::Fern))
    //     .and(&interface.state.cockpit_a_active)
    //     .forward(&interface.systems.lights.abblend);

    // interface
    //     .systems
    //     .cockpit
    //     .beleuchtung_aussen
    //     .process(|sw| *sw == OutsideLightSwitch::Fern)
    //     .and(&interface.state.cockpit_a_active)
    //     .forward(&interface.systems.lights.fern)
    //     .forward(&interface.systems.cockpit.lm_fernlicht);

    interface
        .systems
        .traction
        .direction
        .process(|d| *d == TractionDirection::Backward)
        .and(&interface.state.drive)
        .forward(&interface.systems.lights.rueckfahr);

    interface
        .systems
        .traction
        .target
        .process(|t| *t < 0.0)
        .and(&interface.state.drive)
        .forward(&interface.systems.lights.brems);
}

fn blinker_lights(state: &Interface) {
    let cockpit_a_active = state.state.cockpit_a_active.clone();
    let cockpit_b = state.state.cockpit_b.clone();
    let switch_warnblinker = state.systems.cockpit.warnblinker.clone();
    let switch_blinker_a = state.systems.cockpit.blinker.clone();
    let switch_blinker_b = state.systems.cockpit_rear.blinker.clone();
    let lm_blinker_links = state.systems.cockpit.lm_blinker_links.clone();
    let lm_blinker_rechts = state.systems.cockpit.lm_blinker_rechts.clone();
    let lm_warnblinker = state.systems.cockpit.lm_warnblinker.clone();

    // let blinker_state = state.systems.lights.blinker_state.clone();

    // multiple_on_change(
    //     &[
    //         &cockpit_a_active.clone(),
    //         &cockpit_b.clone(),
    //         &switch_warnblinker.clone(),
    //         &switch_blinker_a.clone(),
    //         &switch_blinker_b.clone(),
    //     ],
    //     move || {
    //         blinker_state.set(if switch_warnblinker.get().is_in() {
    //             BlinkerState::Warning
    //         } else if cockpit_a_active.clone().get() {
    //             match switch_blinker_a.get() {
    //                 BlinkerSwitch::Left => BlinkerState::Left,
    //                 BlinkerSwitch::Right => BlinkerState::Right,
    //                 _ => BlinkerState::Off,
    //             }
    //         } else if cockpit_b.clone().get() {
    //             match switch_blinker_b.get() {
    //                 BlinkerSwitch::Left => BlinkerState::Right,
    //                 BlinkerSwitch::Right => BlinkerState::Left,
    //                 _ => BlinkerState::Off,
    //             }
    //         } else {
    //             BlinkerState::Off
    //         });
    //     },
    // );

    // state
    //     .systems
    //     .lights
    //     .blinker_lampe_rechts
    //     .on_refresh(move |active| {
    //         lm_blinker_rechts.set(*active);
    //     });

    // state
    //     .systems
    //     .lights
    //     .blinker_lampe_links
    //     .on_refresh(move |active| {
    //         lm_blinker_links.set(*active);
    //     });

    // state
    //     .systems
    //     .lights
    //     .lm_warnblinker
    //     .on_refresh(move |active| {
    //         lm_warnblinker.set(*active);
    //     });
}

fn inside_lights(state: &Interface) {
    let cockpit_main = state.systems.lights.cockpit_main.clone();
    let cockpit_begleiter = state.systems.lights.cockpit_begleiter.clone();
    let fahrgastraum = state.systems.lights.fahrgastraum.clone();

    state
        .systems
        .cockpit
        .beleuchtung_fahrerraum
        .on_refresh(move |active| {
            cockpit_main.set(*active >= 2);
            cockpit_begleiter.set(*active >= 1);
        });

    state
        .systems
        .cockpit
        .beleuchtung_fahrgastraum
        .on_refresh(move |active| {
            fahrgastraum.set(*active);
        });
}

async fn door_control(
    doors: DoorsState,
    cockpit: Cockpit,
    passenger: PassengerElementsState,
    traction: TractionState,
) {
    let mut prev_switch_door_1 = false;

    // let shared_doors_closed = Shared::<bool>::default();

    // shared_doors_closed
    //     .delay_relay(0.1, 0.0)
    //     .forward(&cockpit.lm_doors_closed);

    loop {
        let speed = traction.speed.get();
        let door_switch = cockpit.tueren.get();
        let doors_closed = doors.all_closed.get();
        // let released =
        //     (door_switch == DoorSwitch::Released || door_switch == DoorSwitch::Open) && speed < 1.0;

        // Setze alle Status in einem Block
        // let states = {
        //     let all_request = released && door_switch == DoorSwitch::Open;
        //     let switch_door_1 = door_switch == DoorSwitch::Tuer1;
        //     (released, all_request, switch_door_1)
        // };

        // set_var("Door_BtnLgt_Frei", released);

        for (i, (button, request)) in passenger
            .door_buttons
            .iter()
            .zip(doors.requests.iter())
            .enumerate()
        {
            let button_pressed = button.get();
            set_var(
                format!("Door_{}_BtnLgt_Pressed", i + 1).as_str(),
                button_pressed,
            );
            request.set_only_on_change(states.1 || (button_pressed && released));
        }

        // doors.released.set_only_on_change(released);

        // shared_doors_closed.set(!released && doors_closed);

        // doors.vehicle_speed.set_only_on_change(speed);

        // if !prev_switch_door_1 && states.2 {
        //     doors
        //         .door_1_override
        //         .set_only_on_change(match doors.door_1_override.get() {
        //             DoorControlMode::Automatic => DoorControlMode::Open,
        //             DoorControlMode::Open => DoorControlMode::Close,
        //             DoorControlMode::Close => DoorControlMode::Open,
        //         });
        // } else if released {
        //     doors
        //         .door_1_override
        //         .set_only_on_change(DoorControlMode::Automatic);
        // }

        // prev_switch_door_1 = states.2;

        // wait::next_tick().await;
    }
}

fn z_position() {
    let value = Shared::<f32>::var_reader("ZStellung_A")
        .add_shared(&Shared::<f32>::var_reader("ZStellung_B"))
        .multiply_value(4.0);

    InertionSliderState::default()
        .inertion_slider(
            InertionSliderProperties::builder()
                .friction(0.01)
                .additional_force(value)
                .damping_constant(Shared::new(2.0))
                .bumps([
                    Some(
                        InertionSliderBumpProperties::builder()
                            .position(-2.0)
                            .reflection(0.5)
                            .build(),
                    ),
                    Some(
                        InertionSliderBumpProperties::builder()
                            .position(2.0)
                            .reflection(0.5)
                            .build(),
                    ),
                ])
                .build(),
        )
        .position
        .var_writer("ZStellung_C");
}
