use lotus_rt_extra::{
    cockpit_simple::{
        add_button_inout, add_button_twosided_springloaded, add_indicator_light, add_std_button,
        add_step_switch, add_switch, ButtonInOutState, ButtonProperties,
        ButtonTwoSidedSpringLoadedProperties, ButtonTwoSidedSpringLoadedState,
        IndicatorLightProperties, StepSwitchProperties, SwitchProperties,
    },
    drive_control::{add_sollwertgeber, SollwertgeberProperties},
    simple::{add_converter, add_start_stop_sound, StartStopSoundProperties},
    standard_elements::Shared,
};

use crate::cockpit_types::{BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState};

#[derive(Debug, Clone)]
pub struct CockpitState {
    pub lightcheck: Shared<bool>,
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

pub struct CockpitRearState {}

pub fn add_cockpit() -> CockpitState {
    let rw_lock = Shared::new(false);
    let voltage_r = Shared::<f32>::new(1.0);

    let richtungswender = add_step_switch::<RichtungswenderState>(
        StepSwitchProperties::builder()
            .input_event_plus("ReverserPlus")
            .input_event_minus("ReverserMinus")
            .animation_var("A_CP_Richtungswender")
            .position_min(RichtungswenderState::O)
            .position_max(RichtungswenderState::R)
            .blocked(rw_lock.clone())
            .sound("Snd_CP_A_Reverser")
            .build(),
        None::<fn() -> RichtungswenderState>,
        None::<fn() -> RichtungswenderState>,
    );

    let sollwertgeber = add_sollwertgeber(
        SollwertgeberProperties::builder()
            .animation("A_CP_Sollwertgeber")
            .lock(add_converter(
                richtungswender.clone(),
                |state| matches!(state, RichtungswenderState::O | RichtungswenderState::I),
                None,
            ))
            .speed((1.0, 5.0, 20.0))
            .rw_lock(rw_lock.clone())
            .input_events((
                "Throttle".to_string(),
                "Neutral".to_string(),
                "Brake".to_string(),
                "MaxBrake".to_string(),
            ))
            .sounds((
                "Snd_CP_A_SWG_NotchNeutral".to_string(),
                "Snd_CP_A_SWG_End".to_string(),
                "Snd_CP_A_SWG_NotchOther".to_string(),
            ))
            .build(),
    );

    let lm_check = Shared::new(false);

    let add_std_button = |input_event: &str, animation_var: &str| -> Shared<bool> {
        add_std_button(
            ButtonProperties::builder()
                .input_event(input_event)
                .animation_var(animation_var)
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        )
    };

    let add_std_lm = |variable: &str| -> Shared<bool> {
        add_indicator_light(
            IndicatorLightProperties::builder()
                .variable(variable)
                .lighttest(lm_check.clone())
                .voltage(voltage_r.clone())
                .build(),
        )
    };

    let state = CockpitState {
        richtungswender,
        sollwertgeber,
        lm_check: lm_check.clone(),

        sanden: add_std_button("Sanding", "A_CP_TS_Sanden"),
        mg_bremse: add_std_button("RailBrake", "A_CP_TS_MgBremse"),
        klingel: add_std_button("Bell1", "A_CP_TS_Klingel"),
        kinderwagen: add_std_button("ResetBuggy", "A_CP_TS_KiWa"),
        rollstuhl: add_std_button("ResetWheelchair", "A_CP_TS_Rolli"),
        sifa: add_std_button("HoldToRun_Btn", "A_CP_TS_SiFa"),
        lightcheck: add_std_button("Lightcheck", "A_CP_TS_Lampentest"),

        pantograph: add_button_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus("PantographDn")
                .input_event_plus("PantographUp")
                .animation_var("A_CP_SW_Pantograph")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
        hauptschalter: add_button_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus("HighVoltageMainSwitchOff")
                .input_event_plus("HighVoltageMainSwitchOn")
                .animation_var("A_CP_SW_Hauptschalter")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),

        federspeicher_overwrite: add_button_inout(
            ButtonProperties::builder()
                .input_event("FspDeactiveToggle")
                .animation_var("A_CP_TS_Fsp")
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        ),
        beleuchtung_aussen: add_step_switch::<OutsideLightSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus("FrontLightMinus")
                .input_event_plus("FrontLightPlus")
                .position_min(OutsideLightSwitch::Off)
                .position_max(OutsideLightSwitch::Fern)
                .animation_var("A_CP_SW_Aussenbel")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> OutsideLightSwitch>,
            None::<fn() -> OutsideLightSwitch>,
        ),
        blinker: add_step_switch::<BlinkerSwitch>(
            StepSwitchProperties::builder()
                .input_event_minus("IndicatorToLeft")
                .input_event_plus("IndicatorToRight")
                .position_min(BlinkerSwitch::Left)
                .position_max(BlinkerSwitch::Right)
                .animation_var("A_CP_SW_Blinker")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> BlinkerSwitch>,
            None::<fn() -> BlinkerSwitch>,
        ),
        warnblinker: add_button_inout(
            ButtonProperties::builder()
                .input_event("IndicatorWarn")
                .animation_var("A_CP_TS_Warnblinker")
                .sound_on("Snd_CP_A_BtnDn")
                .sound_off("Snd_CP_A_BtnUp")
                .build(),
        ),

        beleuchtung_fahrgastraum: add_switch(
            SwitchProperties::builder()
                .toggle_event("CabinLightToggle")
                .animation_var("A_CP_SW_Innenbel")
                .sound_switch("Snd_CP_A_Switch")
                .build(),
        ),
        beleuchtung_fahrerraum: add_step_switch(
            StepSwitchProperties::builder()
                .input_event_minus("CockpitLightMinus")
                .input_event_plus("CockpitLightPlus")
                .position_min(0)
                .position_max(2)
                .animation_var("A_CP_SW_Fstbel")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),

        tueren: add_step_switch::<DoorSwitch>(
            StepSwitchProperties::builder()
                .input_event_plus("DoorsPlus")
                .input_event_minus("DoorsMinus")
                .position_min(DoorSwitch::Tuer1)
                .position_max(DoorSwitch::Open)
                .position_min_is_springloaded(true)
                .animation_var("A_CP_SW_Tueren")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> DoorSwitch>,
            None::<fn() -> DoorSwitch>,
        ),

        scheibenwischer: add_step_switch(
            StepSwitchProperties::builder()
                .input_event_minus("WiperMinus")
                .input_event_plus("WiperPlus")
                .position_min(0)
                .position_max(3)
                .animation_var("A_CP_SW_Wischer")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),
        sprechstelle: add_button_twosided_springloaded(
            ButtonTwoSidedSpringLoadedProperties::builder()
                .input_event_minus("SprechstelleClear")
                .input_event_plus("SprechstelleSpeak")
                .animation_var("A_CP_SW_Sprechstelle")
                .sound_on("Snd_CP_A_RotBtnOn")
                .sound_off("Snd_CP_A_RotBtnOff")
                .build(),
        ),
        zugbildung: add_step_switch(
            StepSwitchProperties::builder()
                .input_event_minus("ZugbildungMinus")
                .input_event_plus("ZugbildungPlus")
                .position_min(-1)
                .position_max(1)
                .animation_var("A_CP_SW_Zugbildung")
                .sound("Snd_CP_A_Switch")
                .build(),
            None::<fn() -> i8>,
            None::<fn() -> i8>,
        ),

        lm_federspeicher: add_std_lm("A_LM_FSp"),

        lm_fernlicht: add_std_lm("A_LM_Fernlicht"),

        lm_blinker_rechts: add_std_lm("A_LM_BlinkerRechts"),
        lm_blinker_links: add_std_lm("A_LM_BlinkerLinks"),
        lm_warnblinker: add_std_lm("A_LM_Warnblinken"),

        lm_doors_closed: add_std_lm("A_LM_DoorsClosed"),
        lm_haltewunsch: add_std_lm("A_LM_Haltewunsch"),
        lm_kinderwagen: add_std_lm("A_LM_Kinderwagen"),
        lm_rollstuhl: add_std_lm("A_LM_Rollstuhl"),

        lm_schienenbremse: add_std_lm("A_LM_Schienenbremse"),
        lm_sifa: add_std_lm("A_LM_Sifa"),
        lm_sprechstelle: add_std_lm("A_LM_Sprechstelle"),
        lm_hauptschalter: add_std_lm("A_LM_Hauptschalter"),
        lm_notstart: add_std_lm("A_LM_Notstart"),
        lm_notablegen: add_std_lm("A_LM_Notablegen"),
    };

    add_start_stop_sound(
        StartStopSoundProperties::builder()
            .start_sound("Snd_CP_A_DoorsClosed".to_string())
            .set_active(state.lm_doors_closed.clone())
            .build(),
    );

    state
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
