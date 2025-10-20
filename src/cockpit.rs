use lotus_rt_extra::{
    backbone::{ObserverBackbone, VehicleBackbone},
    cockpit_simple::{
        ButtonInOutProperties, ButtonProperties, ButtonTwoSidedSpringLoadedProperties,
        StepSwitchInputToggle, StepSwitchProperties, SwitchProperties, button_inout, std_button,
        step_switch, switch, switch_twosided_springloaded,
    },
    drive_control::{
        SollwertgeberProperties, SollwertgeberPropertiesInputEvents,
        SollwertgeberPropertiesNotchSounds, SollwertgeberPropertiesSpeeds, sollwertgeber,
    },
    input::InputEvent,
    observer::Observer,
};

use crate::{
    backbone_types::{
        Gt6nBlinkerSwitch, Gt6nCockpitInputBools, Gt6nCockpitInputFloats,
        Gt6nCockpitInputInOutState, Gt6nCockpitInputInts,
        Gt6nCockpitInputTwoSidedSpringLoadedState, Gt6nCockpitLeuchtmelder, Gt6nDoorSwitch,
        Gt6nOutsideLightSwitch, Gt6nRichtungswender, SifaPosition,
    },
    cockpit_types::{BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState},
};

pub fn add_cockpit(backbone: &mut VehicleBackbone) {
    let mut voltage_r = Observer::<f32>::default();
    let mut reverser_lock = Observer::<bool>::default();
    let schloss_lock = Observer::<bool>::default();

    let mut schloss = switch(
        SwitchProperties::builder()
            .input_event_on(InputEvent::new("Key_Reverser_L", 0))
            .input_event_off(InputEvent::new("Key_Reverser_R", 0))
            .animation_var("Schluessel_A_RW_turned")
            .standard_position(true)
            .locked(schloss_lock.clone())
            .build(),
    );

    let mut richtungswender = step_switch::<RichtungswenderState>(
        StepSwitchProperties::builder()
            .input_event_plus(InputEvent::new("ReverserPlus", 0))
            .input_event_minus(InputEvent::new("ReverserMinus", 0))
            .animation_var("A_CP_Richtungswender")
            .position_min(RichtungswenderState::O)
            .position_max(RichtungswenderState::R)
            .locked(reverser_lock.or_observer(&mut schloss.not()).clone())
            .sound("Snd_CP_A_Reverser")
            .standard_position(RichtungswenderState::I)
            .build(),
        None::<fn() -> RichtungswenderState>,
        None::<fn() -> RichtungswenderState>,
    );

    backbone.insert(Gt6nRichtungswender, richtungswender.clone());

    let mut richtungswender_locks = richtungswender
        .map(|state| *state == RichtungswenderState::O || *state == RichtungswenderState::I);

    richtungswender_locks.write_to(&reverser_lock);

    backbone.insert(
        Gt6nCockpitInputFloats::Sollwertgeber,
        sollwertgeber(
            SollwertgeberProperties::builder()
                .animation("A_CP_Sollwertgeber")
                .lock(richtungswender_locks)
                .speed(SollwertgeberPropertiesSpeeds {
                    normal: 1.0,
                    high: 5.0,
                    very_high: 20.0,
                })
                .rw_lock(reverser_lock.clone())
                .input_events(SollwertgeberPropertiesInputEvents {
                    throttle: InputEvent::new("Throttle", 0),
                    neutral: InputEvent::new("Neutral", 0),
                    brake: InputEvent::new("Brake", 0),
                    max_brake: InputEvent::new("MaxBrake", 0),
                })
                .sounds(SollwertgeberPropertiesNotchSounds {
                    neutral: "Snd_CP_A_SWG_NotchNeutral".to_string(),
                    end: "Snd_CP_A_SWG_End".to_string(),
                    other: "Snd_CP_A_SWG_NotchOther".to_string(),
                })
                .build(),
        ),
    );

    let mut lm_check = gt6n_button("Lightcheck", "A_CP_TS_Lampentest");

    backbone.insert(
        Gt6nCockpitInputBools::Sifa(SifaPosition::Sollwertgeber),
        std_button(
            ButtonProperties::builder()
                .input_event(InputEvent::new("HoldToRun", 0))
                .animation_var("A_CP_Sollwertgeber_SiFa")
                .sound_on("Snd_CP_A_SWG_SiFa_Dn")
                .sound_off("Snd_CP_A_SWG_SiFa_Up")
                .build(),
        ),
    );

    backbone.insert(
        Gt6nCockpitInputBools::Sanden,
        gt6n_button("Sanding", "A_CP_TS_Sanden"),
    );
    backbone.insert(
        Gt6nCockpitInputBools::MgBremse,
        gt6n_button("RailBrake", "A_CP_TS_MgBremse"),
    );
    backbone.insert(
        Gt6nCockpitInputBools::Klingel,
        gt6n_button("Bell1", "A_CP_TS_Klingel"),
    );
    backbone.insert(
        Gt6nCockpitInputBools::Kinderwagen,
        gt6n_button("ResetBuggy", "A_CP_TS_KiWa"),
    );
    backbone.insert(
        Gt6nCockpitInputBools::Rollstuhl,
        gt6n_button("ResetWheelchair", "A_CP_TS_Rolli"),
    );
    backbone.insert(
        Gt6nCockpitInputBools::Sifa(SifaPosition::Button),
        gt6n_button("HoldToRun_Btn", "A_CP_TS_SiFa"),
    );
    backbone.insert(
        Gt6nCockpitInputTwoSidedSpringLoadedState::Pantograph,
        switch_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus(InputEvent::new("PantographDn", 0))
                .input_event_plus(InputEvent::new("PantographUp", 0))
                .animation_var("A_CP_SW_Pantograph")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
    );
    backbone.insert(
        Gt6nCockpitInputTwoSidedSpringLoadedState::Hauptschalter,
        switch_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus(InputEvent::new("HighVoltageMainSwitchOff", 0))
                .input_event_plus(InputEvent::new("HighVoltageMainSwitchOn", 0))
                .animation_var("A_CP_SW_Hauptschalter")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
    );

    backbone.insert(
        Gt6nCockpitInputInOutState::FederspeicherOverwrite,
        button_inout(
            ButtonInOutProperties::builder()
                .input_event(InputEvent::new("FspDeactiveToggle", 0))
                .animation_var("A_CP_TS_Fsp")
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        ),
    );

    backbone.insert(
        Gt6nOutsideLightSwitch,
        step_switch::<OutsideLightSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("FrontLightMinus", 0))
                .input_event_plus(InputEvent::new("FrontLightPlus", 0))
                .position_min(OutsideLightSwitch::Off)
                .position_max(OutsideLightSwitch::Fern)
                .animation_var("A_CP_SW_Aussenbel")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> OutsideLightSwitch>,
            None::<fn() -> OutsideLightSwitch>,
        ),
    );
    backbone.insert(
        Gt6nBlinkerSwitch,
        step_switch::<BlinkerSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("IndicatorToLeft", 0))
                .input_event_plus(InputEvent::new("IndicatorToRight", 0))
                .input_events_set(vec![
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("IndicatorLeft", 0))
                        .set(BlinkerSwitch::Left)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("IndicatorOff", 0))
                        .set(BlinkerSwitch::Off)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("IndicatorRight", 0))
                        .set(BlinkerSwitch::Right)
                        .build(),
                ])
                .position_min(BlinkerSwitch::Left)
                .position_max(BlinkerSwitch::Right)
                .animation_var("A_CP_SW_Blinker")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> BlinkerSwitch>,
            None::<fn() -> BlinkerSwitch>,
        ),
    );
    backbone.insert(
        Gt6nCockpitInputInOutState::Warnblinker,
        button_inout(
            ButtonInOutProperties::builder()
                .input_event(InputEvent::new("IndicatorWarn", 0))
                .animation_var("A_CP_TS_Warnblinker")
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        ),
    );

    backbone.insert(
        Gt6nCockpitInputBools::BeleuchtungFahrgastraum,
        switch(
            SwitchProperties::builder()
                .input_event_toggle(InputEvent::new("CabinLightToggle", 0))
                .animation_var("A_CP_SW_Innenbel")
                .sound_switch_off("Snd_CP_A_Switch")
                .sound_switch_on("Snd_CP_A_Switch")
                .build(),
        ),
    );
    backbone.insert(
        Gt6nCockpitInputInts::BeleuchtungFahrerraum,
        step_switch(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("CockpitLightMinus", 0))
                .input_event_plus(InputEvent::new("CockpitLightPlus", 0))
                .input_events_set(vec![
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("CockpitLightToggle", 0))
                        .set(2)
                        .set_else(0)
                        .build(),
                ])
                .position_min(0)
                .position_max(2)
                .animation_var("A_CP_SW_Fstbel")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),
    );
    backbone.insert(
        Gt6nDoorSwitch,
        step_switch::<DoorSwitch>(
            StepSwitchProperties::builder()
                .input_event_plus(InputEvent::new("DoorsPlus", 0))
                .input_event_minus(InputEvent::new("DoorsMinus", 0))
                .input_events_set(vec![
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("DoorAllOpen", 0))
                        .set(DoorSwitch::Open)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("DoorAllClose", 0))
                        .set(DoorSwitch::Closed)
                        .build(),
                ])
                .position_min(DoorSwitch::Tuer1)
                .position_max(DoorSwitch::Open)
                .position_min_is_springloaded(true)
                .animation_var("A_CP_SW_Tueren")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> DoorSwitch>,
            None::<fn() -> DoorSwitch>,
        ),
    );
    backbone.insert(
        Gt6nCockpitInputInts::Scheibenwischer,
        step_switch(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("WiperMinus", 0))
                .input_event_plus(InputEvent::new("WiperPlus", 0))
                .position_min(0)
                .position_max(3)
                .animation_var("A_CP_SW_Wischer")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),
    );
    backbone.insert(
        Gt6nCockpitInputTwoSidedSpringLoadedState::Sprechstelle,
        switch_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus(InputEvent::new("SprechstelleClear", 0))
                .input_event_plus(InputEvent::new("SprechstelleSpeak", 0))
                .animation_var("A_CP_SW_Sprechstelle")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
    );
    backbone.insert(
        Gt6nCockpitInputInts::Zugbildung,
        step_switch(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("ZugbildungMinus", 0))
                .input_event_plus(InputEvent::new("ZugbildungPlus", 0))
                .position_min(-1)
                .position_max(1)
                .animation_var("A_CP_SW_Zugbildung")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),
    );

    let mut std_lm = |node_id: Gt6nCockpitLeuchtmelder, variable: &str| -> Observer<bool> {
        let mut value = backbone.add_and_get_new_observer(node_id);
        value
            .or_observer(&mut lm_check)
            .to_float()
            .multiply_observer(&mut voltage_r)
            .var_writer(variable);
        value
    };

    std_lm(Gt6nCockpitLeuchtmelder::Federspeicher, "A_LM_FSp");

    std_lm(Gt6nCockpitLeuchtmelder::Fernlicht, "A_LM_Fernlicht");

    std_lm(Gt6nCockpitLeuchtmelder::BlinkerRechts, "A_LM_BlinkerRechts");
    std_lm(Gt6nCockpitLeuchtmelder::BlinkerLinks, "A_LM_BlinkerLinks");
    std_lm(Gt6nCockpitLeuchtmelder::Warnblinker, "A_LM_Warnblinken");

    let mut lm_doors_closed = std_lm(Gt6nCockpitLeuchtmelder::DoorsClosed, "A_LM_DoorsClosed");
    std_lm(Gt6nCockpitLeuchtmelder::Haltewunsch, "A_LM_Haltewunsch");
    std_lm(Gt6nCockpitLeuchtmelder::Kinderwagen, "A_LM_Kinderwagen");
    std_lm(Gt6nCockpitLeuchtmelder::Rollstuhl, "A_LM_Rollstuhl");

    std_lm(
        Gt6nCockpitLeuchtmelder::Schienenbremse,
        "A_LM_Schienenbremse",
    );
    std_lm(Gt6nCockpitLeuchtmelder::Sifa, "A_LM_Sifa");
    std_lm(Gt6nCockpitLeuchtmelder::Sprechstelle, "A_LM_Sprechstelle");
    std_lm(Gt6nCockpitLeuchtmelder::Hauptschalter, "A_LM_Hauptschalter");
    std_lm(Gt6nCockpitLeuchtmelder::Notstart, "A_LM_Notstart");
    std_lm(Gt6nCockpitLeuchtmelder::Notablegen, "A_LM_Notablegen");

    lm_doors_closed.trigger_sound("Snd_CP_A_DoorsClosed");
}

fn gt6n_button(input_event: &str, animation_var: &str) -> Observer<bool> {
    std_button(
        ButtonProperties::builder()
            .input_event(InputEvent::new(input_event, 0))
            .animation_var(animation_var)
            .sound_on("Snd_CP_A_BtnDn")
            .sound_off("Snd_CP_A_BtnUp")
            .build(),
    )
}
