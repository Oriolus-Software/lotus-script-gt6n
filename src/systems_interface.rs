use lotus_extra::vehicle::CockpitSide;
use lotus_rt_extra::{
    backbone::VehicleBackbone,
    backbone_types,
    cockpit_simple::ButtonInOutState,
    cockpit_special::{TokenProperties, TokenSlot, token},
    doors::DoorControlMode,
    input::InputEvent,
    observer::{Observer, changed_init},
    physic::{InertionSliderBumpProperties, InertionSliderProperties, inertion_slider},
    variables::var_reader,
    vehicle_systems::BlinkerState,
};
use lotus_script::var::set_var;

use crate::{
    backbone_special_types,
    cockpit_types::{
        BackDriveSwitch, BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState,
    },
    traction::TractionDirection,
};

pub fn create_other_observers(backbone: &mut VehicleBackbone) {
    backbone.create_observer(backbone_types::ActiveCockpit);
    backbone.create_observer(backbone_types::DriveMode);
    backbone.create_observer(backbone_types::VehicleSpeed);
    backbone.create_observer(backbone_types::SystemsReady);
    backbone.create_observer(backbone_types::ControlVoltage);
}

pub fn init_interface(backbone: &mut VehicleBackbone) {
    backbone
        .get(backbone_types::SystemsReady)
        .unwrap()
        .call(&true);
    backbone
        .get(backbone_types::ControlVoltage)
        .unwrap()
        .call(&1.0);

    set_var("Coupling_A_casecap", true);
    set_var("Coupling_B_casecap", true);
    set_var("Cfg_OldDisplay", true);
}

pub fn interface(backbone: &mut VehicleBackbone) {
    general_system_active(backbone);
    cockpit(backbone);
    traction_control(backbone);
    doors(backbone);
    outside_lights(backbone);
    blinker_lights(backbone);
    inside_lights(backbone);
    misc(backbone);
    z_position();
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Schluessel {
    Vorne,
    Hinten,
}

fn general_system_active(backbone: &mut VehicleBackbone) {
    if let Some(mut richtungswender) = backbone.get(backbone_special_types::Richtungswender)
        && let Some(ready) = backbone.get(backbone_types::SystemsReady)
        && let Some(mut active_cockpit) = backbone.get(backbone_types::ActiveCockpit)
        && let Some(drive_mode) = backbone.get(backbone_types::DriveMode)
        && let Some(mut schluessel_b) = backbone.get(
            backbone_special_types::CockpitInputBools::Schloss(CockpitSide::B),
        )
    {
        richtungswender
            .map(|state| {
                if *state != RichtungswenderState::O {
                    backbone_types::ActiveCockpitSide::A
                } else {
                    backbone_types::ActiveCockpitSide::Off
                }
            })
            .write_to(&active_cockpit);

        richtungswender
            .map(|state| *state == RichtungswenderState::V || *state == RichtungswenderState::R)
            .write_to(&drive_mode);

        active_cockpit
            .map(|v| *v != backbone_types::ActiveCockpitSide::Off)
            .write_to(&ready);

        schluessel_b
            .write_to(&drive_mode)
            .map(|state| {
                if *state {
                    backbone_types::ActiveCockpitSide::B
                } else {
                    backbone_types::ActiveCockpitSide::Off
                }
            })
            .write_to(&active_cockpit);
    }

    if let Some(mut ready) = backbone.get(backbone_types::SystemsReady) {
        ready.loop_sound("Snd_Cabin_IdleI".to_string());
    }

    if let Some(mut drive) = backbone.get(backbone_types::DriveMode) {
        drive.loop_sound("Snd_Cabin_IdleVR".to_string());
    }
}

fn cockpit(backbone: &mut VehicleBackbone) {
    if let Some(schloss_lock) = backbone.get(
        backbone_special_types::CockpitInputBools::SchlossLock(CockpitSide::A),
    ) {
        token::<Schluessel>(
            TokenProperties::builder()
                .slots(vec![
                    TokenSlot::builder()
                        .token(Schluessel::Vorne)
                        .visibility_var("Schluessel_A_RW")
                        .input_event_set(InputEvent::new("InsertKey_Reverser", 0))
                        .input_event_reset(InputEvent::new("Key_Reverser_R", 0))
                        .sound_set("Snd_CP_A_KeyIn")
                        .sound_reset("Snd_CP_A_KeyOut")
                        .locked_deactivate(schloss_lock.clone())
                        .build(),
                    TokenSlot::builder()
                        .token(Schluessel::Hinten)
                        .visibility_var("Schluessel_H")
                        .input_event_set(InputEvent::new("InsertKey_Reverser", 1))
                        .input_event_reset(InputEvent::new("Key_Reverser_R", 1))
                        .sound_set("Snd_CP_B_KeyIn")
                        .sound_reset("Snd_CP_B_KeyOut")
                        // .locked_deactivate(
                        //     channels_clone
                        //         .cockpit_rear
                        //         .schloss
                        //         .delay_relay(0.0, 0.1)
                        //         .clone(),
                        // )
                        .build(),
                ])
                .build(),
        );
    }
}

fn traction_control(backbone: &mut VehicleBackbone) {
    if let Some(mut ready) = backbone.get(backbone_types::SystemsReady)
        && let Some(mut drive_mode) = backbone.get(backbone_types::DriveMode)
        && let Some(mut active_cockpit) = backbone.get(backbone_types::ActiveCockpit)
        && let Some(direction) = backbone.get(backbone_special_types::TractionDirection)
        && let Some(mut richtungswender) = backbone.get(backbone_special_types::Richtungswender)
        && let Some(mut cockpit_b) = backbone.get(
            backbone_special_types::CockpitInputBools::Schloss(CockpitSide::B),
        )
        && let Some(traction_target) = backbone.get(backbone_special_types::TractionFloat::Target)
        && let Some(mut sollwertgeber) =
            backbone.get(backbone_special_types::CockpitInputFloats::Sollwertgeber)
        && let Some(mut rear_fahrschalter) = backbone.get(backbone_special_types::BackDriveSwitch)
        && let Some(mut btn_mg_bremse) =
            backbone.get(backbone_special_types::CockpitInputBools::MgBremse)
        && let Some(mg_target) = backbone.get(backbone_special_types::TractionBool::MgBremse)
        && let Some(mut federspeicher) =
            backbone.get(backbone_special_types::TractionBool::Federspeicher)
        && let Some(mut btn_federspeicher_overwrite) =
            backbone.get(backbone_special_types::CockpitInputInOutState::FederspeicherOverwrite)
        && let Some(sanden) = backbone.get(backbone_special_types::TractionBool::Sanden)
        && let Some(mut btn_sanden) =
            backbone.get(backbone_special_types::CockpitInputBools::Sanden)
    {
        let mut drive_mode_a = active_cockpit
            .equal_value(backbone_types::ActiveCockpitSide::A)
            .and_observer(&mut drive_mode, false, false);

        let mut drive_mode_b = active_cockpit
            .equal_value(backbone_types::ActiveCockpitSide::B)
            .and_observer(&mut drive_mode, false, false);

        ready
            .tripple_zip(
                &mut richtungswender,
                &mut cockpit_b,
                |(active, richtungswender, cockpit_b), next| {
                    let dir = if *active {
                        match *richtungswender {
                            RichtungswenderState::V => TractionDirection::Forward,
                            RichtungswenderState::R => TractionDirection::Backward,
                            _ => {
                                if *cockpit_b {
                                    TractionDirection::Backward
                                } else {
                                    TractionDirection::Neutral
                                }
                            }
                        }
                    } else {
                        TractionDirection::Neutral
                    };
                    next(&dir);
                },
                false,
                RichtungswenderState::O,
                false,
            )
            .write_to(&direction);

        drive_mode_a
            .zip(
                &mut sollwertgeber,
                |(drive_mode_a, sollwertgeber), next| {
                    if *drive_mode_a {
                        let r = if *sollwertgeber < 0.0 {
                            *sollwertgeber * 1.111
                        } else {
                            *sollwertgeber
                        };
                        next(&r)
                    }
                },
                false,
                0.0,
            )
            .write_to(&traction_target);

        drive_mode_b
            .zip(
                &mut rear_fahrschalter,
                |(drive_mode_b, rear_fahrschalter), next| {
                    let r = if *drive_mode_b {
                        if *rear_fahrschalter == BackDriveSwitch::Drive {
                            0.5
                        } else if *rear_fahrschalter == BackDriveSwitch::Neutral {
                            0.0
                        } else if *rear_fahrschalter == BackDriveSwitch::Brake {
                            -0.6
                        } else {
                            -1.0
                        }
                    } else {
                        0.0
                    };
                    next(&r);
                },
                false,
                BackDriveSwitch::Neutral,
            )
            .write_to(&traction_target);

        btn_mg_bremse
            .and_observer(&mut drive_mode_a, false, false)
            .write_to(&mg_target);

        btn_sanden
            .and_observer(
                &mut active_cockpit.equal_value(backbone_types::ActiveCockpitSide::A),
                false,
                false,
            )
            .write_to(&sanden);

        btn_federspeicher_overwrite
            .map(|v| *v == ButtonInOutState::In)
            .and_observer(
                &mut active_cockpit.equal_value(backbone_types::ActiveCockpitSide::A),
                false,
                false,
            )
            .or_observer(&mut drive_mode.not(), false, true)
            .delay_relay(0.3, 0.3)
            .write_to(&federspeicher);

        if let Some(lm_federspeicher) =
            backbone.get(backbone_special_types::CockpitLeuchtmelder::Federspeicher)
        {
            federspeicher
                .and_observer(
                    &mut active_cockpit.equal_value(backbone_types::ActiveCockpitSide::A),
                    false,
                    false,
                )
                .write_to(&lm_federspeicher);
        }
    }
}

fn outside_lights(backbone: &mut VehicleBackbone) {
    if let Some(mut outside_light_switch) = backbone.get(backbone_special_types::OutsideLightSwitch)
    {
        if let Some(stand) = backbone.get(backbone_special_types::Lights::Stand)
            && let Some(rueck) = backbone.get(backbone_special_types::Lights::Rueck)
            && let Some(instrumente) = backbone.get(backbone_special_types::Lights::Instrumente)
        {
            outside_light_switch
                .not_equal_value(OutsideLightSwitch::Off)
                .write_to(&stand)
                .write_to(&rueck)
                .write_to(&instrumente);
        }

        if let Some(mut active_cockpit) = backbone.get(backbone_types::ActiveCockpit) {
            if let Some(abblend) = backbone.get(backbone_special_types::Lights::Abblend) {
                outside_light_switch
                    .map(|v| *v == OutsideLightSwitch::Abblend || *v == OutsideLightSwitch::Fern)
                    .and_observer(
                        &mut active_cockpit.equal_value(backbone_types::ActiveCockpitSide::A),
                        false,
                        false,
                    )
                    .write_to(&abblend);
            }

            let mut sw_fern = outside_light_switch
                .map(|v| *v == OutsideLightSwitch::Fern)
                .and_observer(
                    &mut active_cockpit.equal_value(backbone_types::ActiveCockpitSide::A),
                    false,
                    false,
                );

            if let Some(fern_outside) = backbone.get(backbone_special_types::Lights::Fern) {
                sw_fern.write_to(&fern_outside);
            }

            if let Some(lm_fernlicht) =
                backbone.get(backbone_special_types::CockpitLeuchtmelder::Fernlicht)
            {
                sw_fern.write_to(&lm_fernlicht);
            }

            if let Some(mut drive_mode_a) = backbone.get(backbone_types::DriveMode) {
                if let Some(rueckfahr) = backbone.get(backbone_special_types::Lights::Rueckfahr)
                    && let Some(mut direction) =
                        backbone.get(backbone_special_types::TractionDirection)
                {
                    direction
                        .map(|d| *d == TractionDirection::Backward)
                        .and_observer(&mut drive_mode_a, false, false)
                        .write_to(&rueckfahr);
                }

                if let Some(mut traction_target) =
                    backbone.get(backbone_special_types::TractionFloat::Target)
                    && let Some(brems) = backbone.get(backbone_special_types::Lights::Brems)
                {
                    traction_target
                        .map(|t| *t < 0.0)
                        .and_observer(&mut drive_mode_a, false, false)
                        .write_to(&brems);
                }
            }
        }
    }
}

fn blinker_lights(backbone: &mut VehicleBackbone) {
    if let Some(blinker_state) = backbone.get(backbone_special_types::LightBlinkerState)
        && let Some(mut switch_warnblinker) =
            backbone.get(backbone_special_types::CockpitInputInOutState::Warnblinker)
        && let Some(mut active_cockpit) = backbone.get(backbone_types::ActiveCockpit)
        && let Some(mut switch_blinker_front) =
            backbone.get(backbone_special_types::BlinkerSwitch::Sw(CockpitSide::A))
        && let Some(mut switch_blinker_back) =
            backbone.get(backbone_special_types::BlinkerSwitch::Sw(CockpitSide::B))
    {
        switch_warnblinker
            .map(|v| *v == ButtonInOutState::Out)
            .if_then_o_v(
                &mut active_cockpit
                    .equal_value(backbone_types::ActiveCockpitSide::A)
                    .if_then_o_o(
                        &mut switch_blinker_front.map(|v| match *v {
                            BlinkerSwitch::Left => BlinkerState::Left,
                            BlinkerSwitch::Right => BlinkerState::Right,
                            _ => BlinkerState::Off,
                        }),
                        &mut active_cockpit
                            .equal_value(backbone_types::ActiveCockpitSide::B)
                            .if_then_o_v(
                                &mut switch_blinker_back.map(|v| match *v {
                                    BlinkerSwitch::Left => BlinkerState::Right,
                                    BlinkerSwitch::Right => BlinkerState::Left,
                                    _ => BlinkerState::Off,
                                }),
                                BlinkerState::Off,
                                false,
                                BlinkerState::Off,
                            ),
                        false,
                        BlinkerState::Off,
                        BlinkerState::Off,
                    ),
                BlinkerState::Warning,
                true,
                BlinkerState::Off,
            )
            .write_to(&blinker_state);
    }

    if let Some(mut blinker_lampe_rechts) =
        backbone.get(backbone_special_types::Lights::BlinkerRechts)
        && let Some(lm_blinker_rechts) = backbone.get(
            backbone_special_types::CockpitLeuchtmelder::BlinkerRechts(CockpitSide::A),
        )
    {
        blinker_lampe_rechts.write_to(&lm_blinker_rechts);
    }

    if let Some(mut blinker_lampe_links) =
        backbone.get(backbone_special_types::Lights::BlinkerLinks)
        && let Some(lm_blinker_links) = backbone.get(
            backbone_special_types::CockpitLeuchtmelder::BlinkerLinks(CockpitSide::A),
        )
    {
        blinker_lampe_links.write_to(&lm_blinker_links);
    }

    if let Some(lm_warnblinker) =
        backbone.get(backbone_special_types::CockpitLeuchtmelder::Warnblinker)
        && let Some(mut warnblinker_light_lm) =
            backbone.get(backbone_special_types::Lights::LmWarnblinker)
    {
        warnblinker_light_lm.write_to(&lm_warnblinker);
    }

    if let Some(mut blinker_lampe_rechts) =
        backbone.get(backbone_special_types::Lights::BlinkerRechts)
        && let Some(lm_blinker_rechts) = backbone.get(
            backbone_special_types::CockpitLeuchtmelder::BlinkerRechts(CockpitSide::B),
        )
    {
        blinker_lampe_rechts.write_to(&lm_blinker_rechts);
    }

    if let Some(mut blinker_lampe_links) =
        backbone.get(backbone_special_types::Lights::BlinkerLinks)
        && let Some(lm_blinker_links) = backbone.get(
            backbone_special_types::CockpitLeuchtmelder::BlinkerLinks(CockpitSide::B),
        )
    {
        blinker_lampe_links.write_to(&lm_blinker_links);
    }
}

fn inside_lights(backbone: &mut VehicleBackbone) {
    if let Some(mut switch_fahrerraum) =
        backbone.get(backbone_special_types::CockpitInputInts::BeleuchtungFahrerraum)
        && let Some(light_fahrerraum) = backbone.get(backbone_special_types::Lights::CockpitMain)
        && let Some(light_begleiter) =
            backbone.get(backbone_special_types::Lights::CockpitBegleiter)
    {
        switch_fahrerraum
            .map(|v| *v >= 2)
            .write_to(&light_fahrerraum);
        switch_fahrerraum
            .map(|v| *v >= 1)
            .write_to(&light_begleiter);
    }

    if let Some(mut switch_fahrgastraum) =
        backbone.get(backbone_special_types::CockpitInputBools::BeleuchtungFahrgastraum)
        && let Some(light_fahrgastraum) = backbone.get(backbone_special_types::Lights::Fahrgastraum)
    {
        switch_fahrgastraum.write_to(&light_fahrgastraum);
    }
}
fn doors(backbone: &mut VehicleBackbone) {
    if let Some(mut door_switch) = backbone.get(backbone_special_types::DoorSwitch)
        && let Some(mut released) = backbone.get(backbone_special_types::DoorsReleased)
        && let Some(mut speed) = backbone.get(backbone_types::VehicleSpeed)
        && let Some(mut all_closed) = backbone.get(backbone_special_types::DoorsAllClosed)
        && let Some(door_1_override) = backbone.get(backbone_special_types::Door1Force)
    {
        let mut switch_released = door_switch.binary(
            &mut speed,
            |d, s| Some((*d == DoorSwitch::Released || *d == DoorSwitch::Open) && *s < 1.0),
            DoorSwitch::Closed,
            0.0,
        );

        let mut all_request = switch_released.and_observer(
            &mut door_switch.equal_value(DoorSwitch::Open),
            false,
            false,
        );
        let mut switch_door_1 = door_switch.equal_value(DoorSwitch::Tuer1);

        switch_released.var_writer("Door_BtnLgt_Frei");

        for i in 0..4 {
            let passenger_door_button = backbone.get(
                backbone_special_types::PassengerDoorButtons::DoorRight(i as i8),
            );
            let door_request =
                backbone.get(backbone_special_types::DoorRequest::DoorRight(i as i8));

            if let Some(door_request) = door_request {
                passenger_door_button
                    .unwrap()
                    .and_observer(&mut switch_released, false, false)
                    .or_observer(&mut all_request, false, false)
                    .write_to(&door_request);
            }
        }

        switch_released.write_to(&released);

        if let Some(lm_doors_closed) =
            backbone.get(backbone_special_types::CockpitLeuchtmelder::DoorsClosed)
        {
            released
                .not()
                .and_observer(&mut all_closed, false, true)
                .delay_relay(0.1, 0.0)
                .write_to(&lm_doors_closed);
        }

        let mut door_1_override_state = DoorControlMode::Close;

        switch_door_1
            .filter_map(move |v| {
                if *v {
                    door_1_override_state = match door_1_override_state {
                        DoorControlMode::Automatic => DoorControlMode::Open,
                        DoorControlMode::Open => DoorControlMode::Close,
                        DoorControlMode::Close => DoorControlMode::Open,
                    };
                    Some(door_1_override_state)
                } else {
                    None
                }
            })
            .write_to(&door_1_override);

        switch_released
            .filter_map(move |v| {
                if *v {
                    Some(DoorControlMode::Automatic)
                } else {
                    None
                }
            })
            .write_to(&door_1_override);
    }
}

fn misc(backbone: &mut VehicleBackbone) {
    if let Some(klingel) = backbone.get(backbone_special_types::MiscBools::Klingel)
        && let Some(mut active_cockpit) = backbone.get(backbone_types::ActiveCockpit)
        && let Some(mut klingel_button) = backbone.get(
            backbone_special_types::CockpitInputBools::Klingel(CockpitSide::A),
        )
        && let Some(mut mg_bremse) =
            backbone.get(backbone_special_types::CockpitInputBools::MgBremse)
    {
        klingel_button
            .or_observer(&mut mg_bremse, false, false)
            .and_observer(
                &mut active_cockpit.equal_value(backbone_types::ActiveCockpitSide::A),
                false,
                false,
            )
            .filter(changed_init(false))
            .write_to(&klingel);
    }
}

fn z_position() {
    let value = var_reader::<f32>("ZStellung_A")
        .add_observer(&mut var_reader::<f32>("ZStellung_B"), 0.0, 0.0)
        .multiply_value(4.0);

    let damping_constant = Observer::<f32>::default();

    inertion_slider(
        InertionSliderProperties::builder()
            .friction(0.01)
            .additional_force(value)
            .damping_constant(damping_constant.clone())
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

    damping_constant.call(&2.0);
}
