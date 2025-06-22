use lotus_rt_extra::{
    cockpit_simple::{
        ButtonInOutProperties, ButtonInOutState, ButtonProperties,
        ButtonTwoSidedSpringLoadedProperties, ButtonTwoSidedSpringLoadedState,
        StepSwitchInputToggle, StepSwitchProperties, SwitchProperties, button_inout, std_button,
        step_switch, switch, switch_twosided_springloaded,
    },
    drive_control::{SollwertgeberProperties, sollwertgeber},
    input::InputEvent,
    shared::Shared,
};

use crate::cockpit_types::{
    BackDriveSwitch, BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState,
};

#[derive(Debug, Clone)]
pub struct Cockpit {
    pub lightcheck: Shared<bool>,
    pub schloss: Shared<bool>,
    pub richtungswender: Shared<RichtungswenderState>,
    pub sollwertgeber: Shared<f32>,
    pub pantograph: Shared<ButtonTwoSidedSpringLoadedState>,
    pub hauptschalter: Shared<ButtonTwoSidedSpringLoadedState>,
    pub federspeicher_overwrite: Shared<ButtonInOutState>,
    pub sanden: Shared<bool>,
    pub mg_bremse: Shared<bool>,
    pub beleuchtung_aussen: Shared<OutsideLightSwitch>,
    pub beleuchtung_fahrerraum: Shared<i8>,
    pub beleuchtung_fahrgastraum: Shared<bool>,
    pub blinker: Shared<BlinkerSwitch>,
    pub warnblinker: Shared<ButtonInOutState>,
    pub klingel: Shared<bool>,
    pub tueren: Shared<DoorSwitch>,
    pub kinderwagen: Shared<bool>,
    pub rollstuhl: Shared<bool>,
    pub sifa: Shared<bool>,
    pub scheibenwischer: Shared<i8>,
    pub sprechstelle: Shared<ButtonTwoSidedSpringLoadedState>,
    pub zugbildung: Shared<i8>,
    pub lm_check: Shared<bool>,
    pub lm_federspeicher: Shared<bool>,
    pub lm_fernlicht: Shared<bool>,
    pub lm_blinker_rechts: Shared<bool>,
    pub lm_blinker_links: Shared<bool>,
    pub lm_warnblinker: Shared<bool>,
    pub lm_doors_closed: Shared<bool>,
    pub lm_haltewunsch: Shared<bool>,
    pub lm_kinderwagen: Shared<bool>,
    pub lm_rollstuhl: Shared<bool>,
    pub lm_schienenbremse: Shared<bool>,
    pub lm_sifa: Shared<bool>,
    pub lm_sprechstelle: Shared<bool>,
    pub lm_hauptschalter: Shared<bool>,
    pub lm_notstart: Shared<bool>,
    pub lm_notablegen: Shared<bool>,
}

#[derive(Debug, Clone)]
pub struct CockpitRear {
    pub schloss: Shared<bool>,
    pub fahrschalter: Shared<BackDriveSwitch>,
    pub klingel: Shared<bool>,
    pub blinker: Shared<BlinkerSwitch>,
    pub tuer_4: Shared<bool>,
    pub lm_blinker_rechts: Shared<bool>,
    pub lm_blinker_links: Shared<bool>,
}

pub fn add_cockpit(voltage_r: Shared<f32>) -> Cockpit {
    let rw_lock = Shared::new(false);
    let schloss_lock = Shared::new(false);

    let schloss = switch(
        SwitchProperties::builder()
            .input_event_on(InputEvent::new("Key_Reverser_L", 0))
            .input_event_off(InputEvent::new("Key_Reverser_R", 0))
            .animation_var("Schluessel_A_RW_turned")
            .standard_position(false)
            .locked(schloss_lock.clone())
            .build(),
    );

    let richtungswender = step_switch::<RichtungswenderState>(
        StepSwitchProperties::builder()
            .input_event_plus(InputEvent::new("ReverserPlus", 0))
            .input_event_minus(InputEvent::new("ReverserMinus", 0))
            .animation_var("A_CP_Richtungswender")
            .position_min(RichtungswenderState::O)
            .position_max(RichtungswenderState::R)
            .locked(rw_lock.or(&schloss.invert()).clone())
            .sound("Snd_CP_A_Reverser")
            .standard_position(RichtungswenderState::I)
            .build(),
        None::<fn() -> RichtungswenderState>,
        None::<fn() -> RichtungswenderState>,
    );

    richtungswender
        .process(|f| !matches!(f, RichtungswenderState::O | RichtungswenderState::I))
        .forward(&schloss_lock);

    let sollwertgeber = sollwertgeber(
        SollwertgeberProperties::builder()
            .animation("A_CP_Sollwertgeber")
            .lock(richtungswender.process(|state| {
                matches!(state, RichtungswenderState::O | RichtungswenderState::I)
            }))
            .speed((1.0, 5.0, 20.0))
            .rw_lock(rw_lock.clone())
            .input_events((
                InputEvent::new("Throttle", 0),
                InputEvent::new("Neutral", 0),
                InputEvent::new("Brake", 0),
                InputEvent::new("MaxBrake", 0),
            ))
            .sounds((
                "Snd_CP_A_SWG_NotchNeutral".to_string(),
                "Snd_CP_A_SWG_End".to_string(),
                "Snd_CP_A_SWG_NotchOther".to_string(),
            ))
            .build(),
    );

    let lm_check = Shared::new(false);

    let gt6n_button = |input_event: &str, animation_var: &str| -> Shared<bool> {
        std_button(
            ButtonProperties::builder()
                .input_event(InputEvent::new(input_event, 0))
                .animation_var(animation_var)
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        )
    };

    let std_lm = |variable: &str| -> Shared<bool> {
        let value = Shared::default();
        value
            .or(&lm_check)
            .to_float()
            .multiply(&voltage_r)
            .var_writer(variable);
        // value.set(false);
        value
    };

    let state = Cockpit {
        richtungswender,
        sollwertgeber,
        lm_check: lm_check.clone(),

        sanden: gt6n_button("Sanding", "A_CP_TS_Sanden"),
        mg_bremse: gt6n_button("RailBrake", "A_CP_TS_MgBremse"),
        klingel: gt6n_button("Bell1", "A_CP_TS_Klingel"),
        kinderwagen: gt6n_button("ResetBuggy", "A_CP_TS_KiWa"),
        rollstuhl: gt6n_button("ResetWheelchair", "A_CP_TS_Rolli"),
        sifa: gt6n_button("HoldToRun_Btn", "A_CP_TS_SiFa"),
        lightcheck: gt6n_button("Lightcheck", "A_CP_TS_Lampentest"),

        pantograph: switch_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus(InputEvent::new("PantographDn", 0))
                .input_event_plus(InputEvent::new("PantographUp", 0))
                .animation_var("A_CP_SW_Pantograph")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
        hauptschalter: switch_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus(InputEvent::new("HighVoltageMainSwitchOff", 0))
                .input_event_plus(InputEvent::new("HighVoltageMainSwitchOn", 0))
                .animation_var("A_CP_SW_Hauptschalter")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),

        federspeicher_overwrite: button_inout(
            ButtonInOutProperties::builder()
                .input_event(InputEvent::new("FspDeactiveToggle", 0))
                .animation_var("A_CP_TS_Fsp")
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        ),
        schloss,
        beleuchtung_aussen: step_switch::<OutsideLightSwitch>(
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
        blinker: step_switch::<BlinkerSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("IndicatorToLeft", 0))
                .input_event_plus(InputEvent::new("IndicatorToRight", 0))
                .position_min(BlinkerSwitch::Left)
                .position_max(BlinkerSwitch::Right)
                .animation_var("A_CP_SW_Blinker")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> BlinkerSwitch>,
            None::<fn() -> BlinkerSwitch>,
        ),
        warnblinker: button_inout(
            ButtonInOutProperties::builder()
                .input_event(InputEvent::new("IndicatorWarn", 0))
                .animation_var("A_CP_TS_Warnblinker")
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        ),

        beleuchtung_fahrgastraum: switch(
            SwitchProperties::builder()
                .input_event_toggle(InputEvent::new("CabinLightToggle", 0))
                .animation_var("A_CP_SW_Innenbel")
                .sound_switch("Snd_CP_A_Switch")
                .build(),
        ),
        beleuchtung_fahrerraum: step_switch(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("CockpitLightMinus", 0))
                .input_event_plus(InputEvent::new("CockpitLightPlus", 0))
                .position_min(0)
                .position_max(2)
                .animation_var("A_CP_SW_Fstbel")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),

        tueren: step_switch::<DoorSwitch>(
            StepSwitchProperties::builder()
                .input_event_plus(InputEvent::new("DoorsPlus", 0))
                .input_event_minus(InputEvent::new("DoorsMinus", 0))
                .position_min(DoorSwitch::Tuer1)
                .position_max(DoorSwitch::Open)
                .position_min_is_springloaded(true)
                .animation_var("A_CP_SW_Tueren")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> DoorSwitch>,
            None::<fn() -> DoorSwitch>,
        ),

        scheibenwischer: step_switch(
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
        sprechstelle: switch_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus(InputEvent::new("SprechstelleClear", 0))
                .input_event_plus(InputEvent::new("SprechstelleSpeak", 0))
                .animation_var("A_CP_SW_Sprechstelle")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
        zugbildung: step_switch(
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

        lm_federspeicher: std_lm("A_LM_FSp"),

        lm_fernlicht: std_lm("A_LM_Fernlicht"),

        lm_blinker_rechts: std_lm("A_LM_BlinkerRechts"),
        lm_blinker_links: std_lm("A_LM_BlinkerLinks"),
        lm_warnblinker: std_lm("A_LM_Warnblinken"),

        lm_doors_closed: std_lm("A_LM_DoorsClosed"),
        lm_haltewunsch: std_lm("A_LM_Haltewunsch"),
        lm_kinderwagen: std_lm("A_LM_Kinderwagen"),
        lm_rollstuhl: std_lm("A_LM_Rollstuhl"),

        lm_schienenbremse: std_lm("A_LM_Schienenbremse"),
        lm_sifa: std_lm("A_LM_Sifa"),
        lm_sprechstelle: std_lm("A_LM_Sprechstelle"),
        lm_hauptschalter: std_lm("A_LM_Hauptschalter"),
        lm_notstart: std_lm("A_LM_Notstart"),
        lm_notablegen: std_lm("A_LM_Notablegen"),
    };

    state.lm_doors_closed.trigger_sound("Snd_CP_A_DoorsClosed");

    state
}

pub fn add_cockpit_rear(voltage_r: Shared<f32>) -> CockpitRear {
    let schloss_lock = Shared::new(false);

    let gt6n_button = |input_event: &str, animation_var: &str| -> Shared<bool> {
        std_button(
            ButtonProperties::builder()
                .input_event(InputEvent::new(input_event, 1))
                .animation_var(animation_var)
                .sound_on("Snd_CP_B_BtnDn")
                .sound_off("Snd_CP_B_BtnUp")
                .build(),
        )
    };

    let std_lm = |variable: &str| -> Shared<bool> {
        let value = Shared::default();
        value.to_float().multiply(&voltage_r).var_writer(variable);
        // value.set(false);
        value
    };

    CockpitRear {
        schloss: switch(
            SwitchProperties::builder()
                .input_event_on(InputEvent::new("Key_Reverser_L", 1))
                .input_event_off(InputEvent::new("Key_Reverser_R", 1))
                .animation_var("Schluessel_H_turned")
                .standard_position(false)
                .locked(schloss_lock.clone())
                .build(),
        ),
        fahrschalter: step_switch(
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
        klingel: gt6n_button("Bell1", "B_CP_TS_Klingel"),
        blinker: step_switch::<BlinkerSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus(InputEvent::new("IndicatorToLeft", 1))
                .input_event_plus(InputEvent::new("IndicatorToRight", 1))
                .position_min(BlinkerSwitch::Left)
                .position_max(BlinkerSwitch::Right)
                .animation_var("B_CP_SW_Blinker")
                .sound("Snd_CP_B_Switch")
                .build(),
            None::<fn() -> BlinkerSwitch>,
            None::<fn() -> BlinkerSwitch>,
        ),
        tuer_4: gt6n_button("Door4Toggle", "B_CP_TS_Tuer4"),
        lm_blinker_rechts: std_lm("B_LM_BlinkerRechts"),
        lm_blinker_links: std_lm("B_LM_BlinkerLinks"),
    }
}

impl From<RichtungswenderState> for i8 {
    fn from(val: RichtungswenderState) -> Self {
        match val {
            RichtungswenderState::O => 0,
            RichtungswenderState::I => 1,
            RichtungswenderState::V => 2,
            RichtungswenderState::R => 3,
        }
    }
}

impl From<i8> for RichtungswenderState {
    fn from(value: i8) -> Self {
        match value {
            1 => RichtungswenderState::I,
            2 => RichtungswenderState::V,
            3 => RichtungswenderState::R,
            _ => RichtungswenderState::O,
        }
    }
}
