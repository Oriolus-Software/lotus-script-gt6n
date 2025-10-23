use crate::{
    backbone_types,
    cockpit_types::{
        BackDriveSwitch, BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState,
    },
};
use lotus_extra::types::CockpitSide;
use lotus_rt_extra::{
    backbone::VehicleBackbone,
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
use strum::IntoEnumIterator;

pub fn add_cockpit(backbone: &mut VehicleBackbone) {
    // Inputs cockpit A ==================================================================================

    let mut voltage_r = backbone
        .get(backbone_types::Voltage)
        .expect("Voltage not found!")
        .clone();

    let sollwertgeber_lock = Observer::<bool>::default();
    let reverser_lock = Observer::<bool>::default();
    let schloss_lock = Observer::<bool>::default();

    backbone.insert(
        backbone_types::CockpitInputBools::SchlossLock(CockpitSide::A),
        Observer::<bool>::default(),
    );

    let mut schloss = switch(
        SwitchProperties::builder()
            .input_event_on(InputEvent::new("Key_Reverser_L", 0))
            .input_event_off(InputEvent::new("Key_Reverser_R", 0))
            .animation_var("Schluessel_A_RW_turned")
            .standard_position(true)
            .locked(schloss_lock.clone())
            .build(),
    );

    backbone.insert(
        backbone_types::CockpitInputBools::Schloss(CockpitSide::A),
        schloss.clone(),
    );

    let mut richtungswender = step_switch::<RichtungswenderState>(
        StepSwitchProperties::builder()
            .input_event_plus(InputEvent::new("ReverserPlus", 0))
            .input_event_minus(InputEvent::new("ReverserMinus", 0))
            .animation_var("A_CP_Richtungswender")
            .position_min(RichtungswenderState::O)
            .position_max(RichtungswenderState::R)
            .locked(reverser_lock.clone())
            .sound("Snd_CP_A_Reverser")
            // .standard_position(RichtungswenderState::I)
            .build(),
        None::<fn() -> RichtungswenderState>,
        None::<fn() -> RichtungswenderState>,
    );

    backbone.insert(backbone_types::Richtungswender, richtungswender.clone());

    backbone.insert(
        backbone_types::CockpitInputFloats::Sollwertgeber,
        sollwertgeber(
            SollwertgeberProperties::builder()
                .animation("A_CP_Sollwertgeber")
                .lock(sollwertgeber_lock.clone())
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

    schloss.not().write_to(&reverser_lock);
    richtungswender
        .map(|state| *state == RichtungswenderState::O || *state == RichtungswenderState::I)
        .write_to(&sollwertgeber_lock);
    richtungswender
        .map(|state| *state == RichtungswenderState::V || *state == RichtungswenderState::R)
        .write_to(&schloss_lock);

    let mut lm_check = gt6n_button("Lightcheck", "A_CP_TS_Lampentest", CockpitSide::A);

    //----

    backbone.insert(
        backbone_types::CockpitInputBools::Sifa(backbone_types::SifaPosition::Sollwertgeber),
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
        backbone_types::CockpitInputBools::Sanden,
        gt6n_button("Sanding", "A_CP_TS_Sanden", CockpitSide::A),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::MgBremse,
        gt6n_button("RailBrake", "A_CP_TS_MgBremse", CockpitSide::A),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::Klingel(CockpitSide::A),
        gt6n_button("Bell1", "A_CP_TS_Klingel", CockpitSide::A),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::Kinderwagen,
        gt6n_button("ResetBuggy", "A_CP_TS_KiWa", CockpitSide::A),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::Rollstuhl,
        gt6n_button("ResetWheelchair", "A_CP_TS_Rolli", CockpitSide::A),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::Sifa(backbone_types::SifaPosition::Button),
        gt6n_button("HoldToRun_Btn", "A_CP_TS_SiFa", CockpitSide::A),
    );
    backbone.insert(
        backbone_types::CockpitInputTwoSidedSpringLoadedState::Pantograph,
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
        backbone_types::CockpitInputTwoSidedSpringLoadedState::Hauptschalter,
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
        backbone_types::CockpitInputInOutState::FederspeicherOverwrite,
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
        backbone_types::OutsideLightSwitch,
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
        backbone_types::BlinkerSwitch::Sw(CockpitSide::A),
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
        backbone_types::CockpitInputInOutState::Warnblinker,
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
        backbone_types::CockpitInputBools::BeleuchtungFahrgastraum,
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
        backbone_types::CockpitInputInts::BeleuchtungFahrerraum,
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
        backbone_types::DoorSwitch,
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
        backbone_types::CockpitInputInts::Scheibenwischer,
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
        backbone_types::CockpitInputTwoSidedSpringLoadedState::Sprechstelle,
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
        backbone_types::CockpitInputInts::Zugbildung,
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

    // Add cockpit B Inputs ==================================================================================

    backbone.insert(
        backbone_types::CockpitInputBools::Schloss(CockpitSide::B),
        switch(
            SwitchProperties::builder()
                .input_event_on(InputEvent::new("Key_Reverser_L", 1))
                .input_event_off(InputEvent::new("Key_Reverser_R", 1))
                .animation_var("Schluessel_H_turned")
                .standard_position(false)
                .locked(schloss_lock.clone())
                .build(),
        ),
    );
    backbone.insert(
        backbone_types::BackDriveSwitch,
        step_switch(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("ThrottleLeaverPlus", 1))
                .input_event_plus(InputEvent::new("ThrottleLeaverMinus", 1))
                .input_events_set(vec![
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("Throttle", 1))
                        .set(BackDriveSwitch::Drive)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("Neutral", 1))
                        .set(BackDriveSwitch::Neutral)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("Brake", 1))
                        .set(BackDriveSwitch::Brake)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("MaxBrake", 1))
                        .set(BackDriveSwitch::MaxBrake)
                        .build(),
                ])
                .position_min(BackDriveSwitch::Drive)
                .position_max(BackDriveSwitch::MaxBrake)
                .animation_var("B_CP_SW_Fahren")
                .sound("Snd_CP_B_Switch")
                .position_min_is_springloaded(true)
                .build(),
            None::<fn() -> BackDriveSwitch>,
            None::<fn() -> BackDriveSwitch>,
        ),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::Klingel(CockpitSide::B),
        gt6n_button("Bell1", "B_CP_TS_Klingel", CockpitSide::B),
    );
    backbone.insert(
        backbone_types::BlinkerSwitch::Sw(CockpitSide::B),
        step_switch::<BlinkerSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("IndicatorToLeft", 1))
                .input_event_plus(InputEvent::new("IndicatorToRight", 1))
                .input_events_set(vec![
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("IndicatorLeft", 1))
                        .set(BlinkerSwitch::Left)
                        .build(),
                    StepSwitchInputToggle::builder()
                        .input_event(InputEvent::new("IndicatorRight", 1))
                        .set(BlinkerSwitch::Right)
                        .build(),
                ])
                .position_min(BlinkerSwitch::Left)
                .position_max(BlinkerSwitch::Right)
                .animation_var("B_CP_SW_Blinker")
                .sound("Snd_CP_B_Switch")
                .build(),
            None::<fn() -> BlinkerSwitch>,
            None::<fn() -> BlinkerSwitch>,
        ),
    );
    backbone.insert(
        backbone_types::CockpitInputBools::Tuer(CockpitSide::B, 4),
        gt6n_button("Door4Toggle", "B_CP_TS_Tuer4", CockpitSide::B),
    );

    // LMs A/B ==================================================================================

    let mut std_lm =
        |node_id: backbone_types::CockpitLeuchtmelder, variable: &str| -> Observer<bool> {
            let mut value = backbone.create_observer(node_id);
            value
                .or_observer(&mut lm_check, false, false)
                .to_float()
                .multiply_observer(&mut voltage_r, 1.0, 0.0)
                .var_writer(variable);
            value
        };

    std_lm(
        backbone_types::CockpitLeuchtmelder::Federspeicher,
        "A_LM_FSp",
    );

    std_lm(
        backbone_types::CockpitLeuchtmelder::Fernlicht,
        "A_LM_Fernlicht",
    );

    std_lm(
        backbone_types::CockpitLeuchtmelder::BlinkerRechts(CockpitSide::A),
        "A_LM_BlinkerRechts",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::BlinkerLinks(CockpitSide::A),
        "A_LM_BlinkerLinks",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Warnblinker,
        "A_LM_Warnblinken",
    );

    let mut lm_doors_closed = std_lm(
        backbone_types::CockpitLeuchtmelder::DoorsClosed,
        "A_LM_DoorsClosed",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Haltewunsch,
        "A_LM_Haltewunsch",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Kinderwagen,
        "A_LM_Kinderwagen",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Rollstuhl,
        "A_LM_Rollstuhl",
    );

    std_lm(
        backbone_types::CockpitLeuchtmelder::Schienenbremse,
        "A_LM_Schienenbremse",
    );
    std_lm(backbone_types::CockpitLeuchtmelder::Sifa, "A_LM_Sifa");
    std_lm(
        backbone_types::CockpitLeuchtmelder::Sprechstelle,
        "A_LM_Sprechstelle",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Hauptschalter,
        "A_LM_Hauptschalter",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Notstart,
        "A_LM_Notstart",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::Notablegen,
        "A_LM_Notablegen",
    );

    lm_doors_closed.trigger_sound("Snd_CP_A_DoorsClosed");

    std_lm(
        backbone_types::CockpitLeuchtmelder::BlinkerRechts(CockpitSide::B),
        "B_LM_BlinkerRechts",
    );
    std_lm(
        backbone_types::CockpitLeuchtmelder::BlinkerLinks(CockpitSide::B),
        "B_LM_BlinkerLinks",
    );

    // Initialize =====================================================================================

    // richtungswender_locks.call(&true);
    // lm_check.call(&false);

    backbone_types::CockpitLeuchtmelder::iter().for_each(|lm| {
        if let Some(a) = backbone.get(lm) {
            a.call(&false);
        }
    });
}

fn gt6n_button(
    input_event: &str,
    animation_var: &str,
    cockpit_side: CockpitSide,
) -> Observer<bool> {
    std_button(
        ButtonProperties::builder()
            .input_event(InputEvent::new(input_event, cockpit_side.into()))
            .animation_var(animation_var)
            .sound_on("Snd_CP_A_BtnDn")
            .sound_off("Snd_CP_A_BtnUp")
            .build(),
    )
}
