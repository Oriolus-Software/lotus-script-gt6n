use lotus_rt::{spawn, wait};
use lotus_script::var::VariableType;

use crate::{
    cockpit::{CockpitState, RichtungswenderState},
    doors::DoorsState,
    lights::{BlinkerSwitch, LightState},
    misc::MiscState,
    passenger_elements::PassengerElementsState,
    standard_elements::{multiple_on_change, Shared},
    tech_elements::{
        doors::DoorControlMode,
        simple::{
            add_and, add_converter, add_delay_relay, add_loop_sound, add_or, DelayRelayProperties,
            LoopSoundProperties,
        },
    },
    traction::{TractionDirection, TractionState},
};

#[derive(Debug, Clone)]
pub struct SystemStates {
    pub cockpit: CockpitState,
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
}

#[derive(Clone)]
struct Interface {
    systems: SystemStates,
    interface: InterfaceState,
}

pub fn add_systems_interface(channels: SystemStates) {
    let channels_clone = channels.clone();
    let state = Interface {
        systems: channels,
        interface: InterfaceState {
            cockpit_a_active: add_converter(
                channels_clone.cockpit.richtungswender.clone(),
                |r| !matches!(r, RichtungswenderState::O),
                None,
            ),
            cockpit_a_drive: add_converter(
                channels_clone.cockpit.richtungswender.clone(),
                |r| matches!(r, RichtungswenderState::V | RichtungswenderState::R),
                None,
            ),
        },
    };

    state.systems.lights.voltage.set(1.0);

    traction_control(&state);

    spawn(federspeicher(
        state.systems.cockpit.clone(),
        state.systems.traction.clone(),
        state.interface.clone(),
    ));

    spawn(sanding_unit(state.clone()));

    outside_lights(&state);
    blinker_lights(&state);

    inside_lights(&state);

    // Cockpit ---------------------------------------

    add_and(
        vec![
            add_or(
                vec![
                    state.systems.cockpit.mg_bremse.clone(),
                    state.systems.cockpit.klingel.clone(),
                ],
                None,
            ),
            state.interface.cockpit_a_active.clone(),
        ],
        Some(&state.systems.misc.klingel),
    );

    add_and(
        vec![
            state.systems.traction.federspeicher.clone(),
            state.interface.cockpit_a_active.clone(),
        ],
        Some(&state.systems.cockpit.lm_federspeicher),
    );

    add_and(
        vec![
            state.systems.cockpit.lightcheck.clone(),
            state.interface.cockpit_a_active.clone(),
        ],
        Some(&state.systems.cockpit.lm_check.clone()),
    );

    // Doors ---------------------------------------

    spawn(door_control(
        state.systems.doors.clone(),
        state.systems.cockpit.clone(),
        state.systems.passenger.clone(),
        state.systems.traction.clone(),
    ));

    // Misc Systems ---------------------------------------

    add_loop_sound(
        LoopSoundProperties::builder()
            .loop_sound("Snd_Cabin_IdleI".to_string())
            .set_active(state.interface.cockpit_a_active.clone())
            .build(),
    );

    add_loop_sound(
        LoopSoundProperties::builder()
            .loop_sound("Snd_Cabin_IdleVR".to_string())
            .set_active(state.interface.cockpit_a_drive.clone())
            .build(),
    );
}

async fn federspeicher(cockpit: CockpitState, traction: TractionState, interface: InterfaceState) {
    let mut prev = false;
    loop {
        let new_value = !interface.cockpit_a_drive.get()
            || (interface.cockpit_a_active.get() && cockpit.federspeicher_overwrite.get());

        if prev != new_value {
            wait::seconds(0.3).await;
            traction.federspeicher.set(new_value);
        }

        prev = new_value;

        wait::next_tick().await;
    }
}

async fn sanding_unit(state: Interface) {
    let mut prev = false;
    loop {
        let new_value =
            state.systems.cockpit.sanden.get() && state.interface.cockpit_a_active.get();

        if prev != new_value {
            state.systems.traction.sanding.set(new_value);
            prev = new_value;
        }

        wait::next_tick().await;
    }
}

fn traction_control(state: &Interface) {
    let cockpit_a_active = state.interface.cockpit_a_active.clone();

    let direction = state.systems.traction.direction.clone();
    let richtungswender = state.systems.cockpit.richtungswender.clone();

    multiple_on_change(
        &[&cockpit_a_active.clone(), &richtungswender.clone()],
        move || {
            direction.set(if cockpit_a_active.get() {
                match richtungswender.get() {
                    RichtungswenderState::V => TractionDirection::Forward,
                    RichtungswenderState::R => TractionDirection::Backward,
                    _ => TractionDirection::Neutral,
                }
            } else {
                TractionDirection::Neutral
            });
        },
    );

    let cockpit_a_active = state.interface.cockpit_a_active.clone();

    let sollwertgeber = state.systems.cockpit.sollwertgeber.clone();
    let traction_target = state.systems.traction.target.clone();

    multiple_on_change(
        &[&cockpit_a_active.clone(), &sollwertgeber.clone()],
        move || {
            traction_target.set(if cockpit_a_active.get() {
                if sollwertgeber.get() < 0.0 {
                    sollwertgeber.get() * 1.111
                } else {
                    sollwertgeber.get()
                }
            } else {
                0.0
            });
        },
    );

    let cockpit_a_active = state.interface.cockpit_a_active.clone();
    let cockpit_mg_bremse = state.systems.cockpit.mg_bremse.clone();
    let mg_target = state.systems.traction.mg.clone();

    multiple_on_change(
        &[&cockpit_a_active.clone(), &cockpit_mg_bremse.clone()],
        move || {
            mg_target.set(cockpit_a_active.get() && cockpit_mg_bremse.get());
        },
    );

    2.3.set("v_Axle_mps_0_1_abs");
    2.4.set("abs");
}

fn outside_lights(state: &Interface) {
    let cockpit_a_active = state.interface.cockpit_a_active.clone();
    let switch_aussen = state.systems.cockpit.beleuchtung_aussen.clone();

    let richtungswender = state.systems.cockpit.richtungswender.clone();
    let sollwertgeber = state.systems.cockpit.sollwertgeber.clone();

    let instrumente = state.systems.lights.instrumente.clone();
    let lm_fernlicht = state.systems.cockpit.lm_fernlicht.clone();

    let standlicht = state.systems.lights.stand.clone();
    let ruecklicht = state.systems.lights.rueck.clone();
    let abblend = state.systems.lights.abblend.clone();
    let fern = state.systems.lights.fern.clone();
    let rueckfahr = state.systems.lights.rueckfahr.clone();
    let brems = state.systems.lights.brems.clone();

    multiple_on_change(
        &[&switch_aussen.clone(), &cockpit_a_active.clone()],
        move || {
            let active = cockpit_a_active.get();
            let switch_aussen = switch_aussen.get();
            let switch_standlicht = switch_aussen > 0;

            standlicht.set(switch_standlicht);
            ruecklicht.set(switch_standlicht);
            instrumente.set(switch_standlicht);

            abblend.set(switch_aussen > 1 && active);
            fern.set(switch_aussen > 2 && active);
            lm_fernlicht.set(switch_aussen > 2 && active);
            rueckfahr.set(richtungswender.get() == RichtungswenderState::R);
            brems.set(sollwertgeber.get() < 0.0);
        },
    );
}

fn blinker_lights(state: &Interface) {
    let cockpit_a_active = state.interface.cockpit_a_active.clone();
    let switch_warnblinker = state.systems.cockpit.warnblinker.clone();
    let switch_blinker = state.systems.cockpit.blinker.clone();
    let lm_blinker_links = state.systems.cockpit.lm_blinker_links.clone();
    let lm_blinker_rechts = state.systems.cockpit.lm_blinker_rechts.clone();
    let lm_warnblinker = state.systems.cockpit.lm_warnblinker.clone();

    let blinker_state = state.systems.lights.blinker_state.clone();

    multiple_on_change(
        &[
            &cockpit_a_active.clone(),
            &switch_warnblinker.clone(),
            &switch_blinker.clone(),
        ],
        move || {
            blinker_state.set(if switch_warnblinker.get() {
                BlinkerSwitch::Warn
            } else if cockpit_a_active.clone().get() {
                match switch_blinker.get() {
                    0 => BlinkerSwitch::Links,
                    2 => BlinkerSwitch::Rechts,
                    _ => BlinkerSwitch::Aus,
                }
            } else {
                BlinkerSwitch::Aus
            });
        },
    );

    state.systems.lights.blinker_lampe_rechts.on_change(
        move |active| {
            lm_blinker_rechts.set(*active);
        },
        "blinker_lampe_rechts".to_string(),
    );

    state.systems.lights.blinker_lampe_links.on_change(
        move |active| {
            lm_blinker_links.set(*active);
        },
        "blinker_lampe_links".to_string(),
    );

    state.systems.lights.lm_warnblinker.on_change(
        move |active| {
            lm_warnblinker.set(*active);
        },
        "blinker_for_lm_warnblinker".to_string(),
    );
}

fn inside_lights(state: &Interface) {
    let cockpit_main = state.systems.lights.cockpit_main.clone();
    let cockpit_begleiter = state.systems.lights.cockpit_begleiter.clone();
    let fahrgastraum = state.systems.lights.fahrgastraum.clone();

    state.systems.cockpit.beleuchtung_fahrerraum.on_change(
        move |active| {
            cockpit_main.set(*active >= 2);
            cockpit_begleiter.set(*active >= 1);
        },
        "switch_fahrerraum".to_string(),
    );

    state.systems.cockpit.beleuchtung_fahrgastraum.on_change(
        move |active| {
            fahrgastraum.set(*active);
        },
        "switch_fahrgastraum".to_string(),
    );
}

async fn door_control(
    doors: DoorsState,
    cockpit: CockpitState,
    passenger: PassengerElementsState,
    traction: TractionState,
) {
    let mut prev_switch_door_1 = false;

    let shared_doors_closed = Shared::<bool>::default();

    add_delay_relay(
        DelayRelayProperties {
            on_delay: 0.1,
            off_delay: 0.0,
            set: shared_doors_closed.clone(),
        },
        Some(&cockpit.lm_doors_closed.clone()),
    );

    loop {
        let speed = traction.speed.get();
        let door_switch = cockpit.tueren.get();
        let doors_closed = doors.all_closed.get();
        let released = door_switch > 0 && speed < 1.0;

        // Setze alle Status in einem Block
        let states = {
            let all_request = released && door_switch == 2;
            let switch_door_1 = door_switch < 0;
            (released, all_request, switch_door_1)
        };

        released.set("Door_BtnLgt_Frei");

        for (i, (button, request)) in passenger
            .door_buttons
            .iter()
            .zip(doors.requests.iter())
            .enumerate()
        {
            let button_pressed = button.get();
            button_pressed.set(format!("Door_{}_BtnLgt_Pressed", i + 1).as_str());
            request.set_only_on_change(states.1 || (button_pressed && released));
        }

        doors.released.set_only_on_change(released);

        shared_doors_closed.set(!released && doors_closed);

        // if prev_doors_closed != lm_green && lm_green {
        //     true.set("Snd_CP_A_DoorsClosed");
        // }
        // prev_doors_closed = lm_green;

        doors.vehicle_speed.set_only_on_change(speed);

        if !prev_switch_door_1 && states.2 {
            doors
                .door_1_override
                .set_only_on_change(match doors.door_1_override.get() {
                    DoorControlMode::Automatic => DoorControlMode::Open,
                    DoorControlMode::Open => DoorControlMode::Close,
                    DoorControlMode::Close => DoorControlMode::Open,
                });
        } else if released {
            doors
                .door_1_override
                .set_only_on_change(DoorControlMode::Automatic);
        }

        prev_switch_door_1 = states.2;

        wait::next_tick().await;
    }
}
