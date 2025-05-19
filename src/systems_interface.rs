use lotus_rt::{spawn, wait};
use lotus_rt_extra::{
    doors::DoorControlMode,
    shared::{multiple_on_change, Shared},
};
use lotus_script::{log, var::set_var};

use crate::{
    cockpit::CockpitState,
    cockpit_types::{BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState},
    doors::DoorsState,
    lights::{BlinkerState, LightState},
    misc::MiscState,
    passenger_elements::PassengerElementsState,
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

pub fn systems_interface(channels: SystemStates) {
    let channels_clone = channels.clone();
    let state = Interface {
        systems: channels,
        interface: InterfaceState {
            cockpit_a_active: channels_clone
                .cockpit
                .richtungswender
                .process(|r| !matches!(r, RichtungswenderState::O), false),
            cockpit_a_drive: channels_clone.cockpit.richtungswender.process(
                |r| matches!(r, RichtungswenderState::V | RichtungswenderState::R),
                false,
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

    state
        .systems
        .cockpit
        // .klingel
        .mg_bremse
        .or(&state.systems.cockpit.klingel)
        .and(&state.interface.cockpit_a_active)
        .forward(&state.systems.misc.klingel);

    state
        .systems
        .cockpit
        .lightcheck
        .and(&state.interface.cockpit_a_active)
        .forward(&state.systems.cockpit.lm_check);

    state
        .systems
        .traction
        .federspeicher
        .and(&state.interface.cockpit_a_active)
        .forward(&state.systems.cockpit.lm_federspeicher);

    // Doors ---------------------------------------

    spawn(door_control(
        state.systems.doors.clone(),
        state.systems.cockpit.clone(),
        state.systems.passenger.clone(),
        state.systems.traction.clone(),
    ));

    // Misc Systems ---------------------------------------

    state
        .interface
        .cockpit_a_active
        .loop_sound("Snd_Cabin_IdleI".to_string());

    state
        .interface
        .cockpit_a_drive
        .loop_sound("Snd_Cabin_IdleVR".to_string());
}

async fn federspeicher(cockpit: CockpitState, traction: TractionState, interface: InterfaceState) {
    let mut prev = false;
    loop {
        let new_value = !interface.cockpit_a_drive.get()
            || (interface.cockpit_a_active.get() && cockpit.federspeicher_overwrite.get().is_in());

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

    set_var("v_Axle_mps_0_1_abs", &2.3);
    set_var("abs", &true);
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
            let switch_standlicht = switch_aussen != OutsideLightSwitch::Off;
            let switch_abblend = (switch_aussen == OutsideLightSwitch::Abblend)
                || (switch_aussen == OutsideLightSwitch::Fern);
            let switch_fern = switch_aussen == OutsideLightSwitch::Fern;

            log::info!("switch_abblend: {}", switch_abblend);

            standlicht.set(switch_standlicht);
            ruecklicht.set(switch_standlicht);
            instrumente.set(switch_standlicht);

            abblend.set(switch_abblend && active);
            fern.set(switch_fern && active);
            lm_fernlicht.set(switch_fern && active);
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
            blinker_state.set(if switch_warnblinker.get().is_in() {
                BlinkerState::Warn
            } else if cockpit_a_active.clone().get() {
                match switch_blinker.get() {
                    BlinkerSwitch::Left => BlinkerState::Links,
                    BlinkerSwitch::Right => BlinkerState::Rechts,
                    _ => BlinkerState::Aus,
                }
            } else {
                BlinkerState::Aus
            });
        },
    );

    state
        .systems
        .lights
        .blinker_lampe_rechts
        .on_refresh(move |active| {
            lm_blinker_rechts.set(*active);
        });

    state
        .systems
        .lights
        .blinker_lampe_links
        .on_refresh(move |active| {
            lm_blinker_links.set(*active);
        });

    state
        .systems
        .lights
        .lm_warnblinker
        .on_refresh(move |active| {
            lm_warnblinker.set(*active);
        });
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
    cockpit: CockpitState,
    passenger: PassengerElementsState,
    traction: TractionState,
) {
    let mut prev_switch_door_1 = false;

    let shared_doors_closed = Shared::<bool>::default();

    shared_doors_closed
        .delay_relay(0.1, 0.0)
        .forward(&cockpit.lm_doors_closed);

    loop {
        let speed = traction.speed.get();
        let door_switch = cockpit.tueren.get();
        let doors_closed = doors.all_closed.get();
        let released =
            (door_switch == DoorSwitch::Released || door_switch == DoorSwitch::Open) && speed < 1.0;

        // Setze alle Status in einem Block
        let states = {
            let all_request = released && door_switch == DoorSwitch::Open;
            let switch_door_1 = door_switch == DoorSwitch::Tuer1;
            (released, all_request, switch_door_1)
        };

        set_var("Door_BtnLgt_Frei", &released);

        for (i, (button, request)) in passenger
            .door_buttons
            .iter()
            .zip(doors.requests.iter())
            .enumerate()
        {
            let button_pressed = button.get();
            set_var(
                format!("Door_{}_BtnLgt_Pressed", i + 1).as_str(),
                &button_pressed,
            );
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
