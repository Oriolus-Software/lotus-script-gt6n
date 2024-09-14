use crate::{
    standard_elements::Shared,
    tech_elements::{
        add_button, add_indicator_light, ButtonProperties, ButtonTwoSidedSpringLoadedProperties,
        IndicatorLightProperties,
    },
};
use lotus_rt::spawn;
use lotus_script::{time, var::VariableType};

use crate::tech_elements::add_button_twosided_springloaded;

#[derive(Debug, Clone)]
pub struct ChannelsCockpit {
    pub richtungswender_r: Shared<RichtungswenderState>,
    pub sollwertgeber_r: Shared<f32>,
    pub federspeicher_overwrite_r: Shared<bool>,
    pub federspeicher_t: Shared<bool>,
}

pub fn add_cockpit() -> ChannelsCockpit {
    let rw_lock = Shared::new(false);
    let voltage_r = Shared::<f32>::new(1.0);

    let richtungswender_r = add_richtungswender(rw_lock.clone());
    let sollwertgeber_r = add_sollwertgeber(richtungswender_r.clone(), rw_lock.clone());

    add_button_twosided_springloaded(
        ButtonTwoSidedSpringLoadedProperties::builder()
            .input_event_minus("HighVoltageMainSwitchOn")
            .input_event_plus("HighVoltageMainSwitchOff")
            .animation_var("A_CP_SW_Hauptschalter")
            .sound_on("Snd_CP_A_RotBtnOn")
            .sound_off("Snd_CP_A_RotBtnOff")
            .build(),
    );

    let lightcheck = add_button(
        ButtonProperties::builder()
            .input_event("Lightcheck")
            .animation_var("A_CP_TS_Lampentest")
            .sound_on("Snd_CP_A_BtnDn")
            .sound_off("Snd_CP_A_BtnUp")
            .build(),
    );

    let federspeicher_overwrite_r = add_button(ButtonProperties {
        input_event: "FspDeactiveToggle".into(),
        animation_var: Some("A_CP_TS_Fsp".into()),
        sound_on: Some("Snd_CP_A_BtnDn".into()),
        sound_off: Some("Snd_CP_A_BtnUp".into()),
    });

    let federspeicher_t = add_indicator_light(
        IndicatorLightProperties::builder()
            .variable("A_LM_FSp")
            .lighttest(lightcheck.clone())
            .voltage(voltage_r)
            .build(),
    );

    ChannelsCockpit {
        richtungswender_r,
        sollwertgeber_r,
        federspeicher_overwrite_r,
        federspeicher_t,
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RichtungswenderState {
    #[default]
    O,
    I,
    V,
    R,
}

pub fn add_richtungswender(lock: Shared<bool>) -> Shared<RichtungswenderState> {
    let state = Shared::new(RichtungswenderState::O);

    fn angle(state: RichtungswenderState) -> f32 {
        match state {
            RichtungswenderState::I => 29.0,
            RichtungswenderState::V => 58.0,
            RichtungswenderState::R => 135.0,
            _ => 0.0,
        }
    }

    let state_clone = state.clone();

    let lock_clone = lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("ReverserPlus").await;

            if lock_clone.get() {
                continue;
            }

            let mut state_new = None;

            match state_clone.get() {
                RichtungswenderState::O => state_new = Some(RichtungswenderState::I),
                RichtungswenderState::I => state_new = Some(RichtungswenderState::V),
                RichtungswenderState::V => state_new = Some(RichtungswenderState::R),
                _ => {}
            }

            if let Some(n) = state_new {
                state_clone.set(n);
                angle(n).set("A_CP_Richtungswender");
                true.set("Snd_CP_A_Reverser");
            }
        }
    });

    let state_clone = state.clone();

    let lock_clone = lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("ReverserMinus").await;

            if lock_clone.get() {
                continue;
            }

            let mut sn = None;

            match state_clone.get() {
                RichtungswenderState::I => sn = Some(RichtungswenderState::O),
                RichtungswenderState::V => sn = Some(RichtungswenderState::I),
                RichtungswenderState::R => sn = Some(RichtungswenderState::V),
                _ => {}
            }

            if let Some(n) = sn {
                state_clone.set(n);
                angle(n).set("A_CP_Richtungswender");
                true.set("Snd_CP_A_Reverser");
            }
        }
    });

    state
}

pub fn add_sollwertgeber(
    richtungswender: Shared<RichtungswenderState>,
    rw_lock: Shared<bool>,
) -> Shared<f32> {
    const SPEED: f32 = 1.0;
    const SPEED_HIGH: f32 = 5.0;
    const SPEED_VERYHIGH: f32 = 20.0;

    let position = Shared::new(0.0);
    let speed = Shared::new(0.0);
    let target = Shared::new(0.0f32);
    let mode = Shared::new(SollwertgeberMode::Neutral);
    let notch = Shared::new(SollwertgeberNotch::Neutral);

    let p = position.clone();
    let sp = speed.clone();
    let md = mode.clone();
    let tg = target.clone();
    spawn(async move {
        loop {
            lotus_rt::select! {
                _ = lotus_rt::wait::just_released("Throttle")=>{}
                _ = lotus_rt::wait::just_released("Neutral")=>{}
                _ = lotus_rt::wait::just_released("Brake")=>{}
            }
            let pos = p.get();
            let not_near_neutral = !(-0.1..0.1).contains(&pos);
            match md.get() {
                SollwertgeberMode::Throttle | SollwertgeberMode::Brake => {
                    if not_near_neutral {
                        tg.set(pos);
                        sp.set(0.0);
                    } else if tg.get() > 0.0 {
                        tg.set(0.1);
                    } else {
                        tg.set(-0.1);
                    }
                }
                _ => {}
            }
            rw_lock.set(not_near_neutral);
        }
    });

    let p = position.clone();
    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let rw = richtungswender.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("Brake").await;

            match rw.get() {
                RichtungswenderState::R | RichtungswenderState::V => {
                    let pos = p.get();
                    let mode = md.get();

                    match mode {
                        SollwertgeberMode::Throttle => {
                            if pos > 0.15 {
                                sp.set(-SPEED);
                                tg.set(0.1);
                            } else {
                                sp.set(-SPEED_HIGH);
                                tg.set(0.0);
                                md.set(SollwertgeberMode::Neutral);
                            }
                        }
                        SollwertgeberMode::Neutral => {
                            sp.set(-SPEED);
                            tg.set(-0.9);
                            md.set(SollwertgeberMode::Brake);
                        }
                        SollwertgeberMode::Brake => {
                            if pos > -0.9 {
                                sp.set(-SPEED);
                                tg.set(-0.9);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    let p = position.clone();
    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let rw = richtungswender.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("Throttle").await;

            match rw.get() {
                RichtungswenderState::R | RichtungswenderState::V => {
                    let pos = p.get();
                    let mode = md.get();

                    match mode {
                        SollwertgeberMode::Throttle => {
                            sp.set(SPEED);
                            tg.set(1.0);
                        }
                        SollwertgeberMode::Neutral => {
                            sp.set(SPEED);
                            tg.set(1.0);
                            md.set(SollwertgeberMode::Throttle);
                        }

                        SollwertgeberMode::Brake | SollwertgeberMode::EmergencyBrake => {
                            if pos < -0.15 {
                                sp.set(SPEED);
                                tg.set(-0.1);
                                if pos < -0.9 {
                                    p.set(-0.9);
                                }
                                md.set(SollwertgeberMode::Brake);
                            } else {
                                sp.set(SPEED_HIGH);
                                tg.set(0.0);
                                md.set(SollwertgeberMode::Neutral);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    let p = position.clone();
    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("Neutral").await;

            if p.get() > 0.0 {
                sp.set(-SPEED_HIGH);
            } else {
                sp.set(SPEED_HIGH);
            }
            tg.set(0.0);
            md.set(SollwertgeberMode::Neutral);
        }
    });

    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let rw = richtungswender.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("MaxBrake").await;

            match rw.get() {
                RichtungswenderState::R | RichtungswenderState::V => {
                    sp.set(-SPEED_VERYHIGH);
                    tg.set(-1.0);
                    md.set(SollwertgeberMode::EmergencyBrake);
                }
                _ => {}
            }
        }
    });

    fn calc_sounds(pos: Shared<f32>, notch: Shared<SollwertgeberNotch>) {
        let pos = pos.get();
        let old_notch = notch.get();

        let new_notch = if (-0.95..-0.87).contains(&pos) {
            SollwertgeberNotch::MaxBrake
        } else if (-0.87..-0.13).contains(&pos) {
            SollwertgeberNotch::Brake
        } else if (-0.13..-0.05).contains(&pos) {
            SollwertgeberNotch::MinBrake
        } else if (-0.05..0.05).contains(&pos) {
            SollwertgeberNotch::Neutral
        } else if (0.05..0.13).contains(&pos) {
            SollwertgeberNotch::MinThrottle
        } else if (0.13..0.97).contains(&pos) {
            SollwertgeberNotch::Throttle
        } else if (0.97..).contains(&pos) {
            SollwertgeberNotch::MaxTrottle
        } else {
            SollwertgeberNotch::EmergencyBrake
        };

        if old_notch != new_notch {
            if new_notch == SollwertgeberNotch::Neutral {
                true.set("Snd_CP_A_SWG_NotchNeutral");
            } else if ((old_notch == SollwertgeberNotch::Throttle)
                && (new_notch == SollwertgeberNotch::MaxTrottle))
                || ((old_notch == SollwertgeberNotch::Brake)
                    && (new_notch == SollwertgeberNotch::MaxBrake))
            {
                true.set("Snd_CP_A_SWG_End");
            } else if !(((old_notch == SollwertgeberNotch::MaxTrottle)
                && (new_notch == SollwertgeberNotch::Throttle))
                || ((old_notch == SollwertgeberNotch::MaxBrake)
                    && (new_notch == SollwertgeberNotch::Brake)))
            {
                true.set("Snd_CP_A_SWG_NotchOther");
            }

            notch.set(new_notch);
        }
    }

    let p = position.clone();
    let n = notch.clone();
    spawn(async move {
        loop {
            let speed_val = speed.get();
            let target_val = target.get();

            // Das funktioniert nicht, solange die Position auch direkt gesetzt werden kann (z.B. per "Neutral")
            // if speed != 0.0 {
            let mut position = p.get() + speed_val * time::delta();

            let pos_max = target_val.min(1.0f32);
            let pos_min = target_val.max(-1.0f32);
            if (speed_val > 0.0) && (position > pos_max) {
                position = pos_max;
                speed.set(0.0);
            }
            if (speed_val < 0.0) && (position < pos_min) {
                position = pos_min;
                speed.set(0.0);
            }

            p.set(position);
            position.set("A_CP_Sollwertgeber");

            calc_sounds(p.clone(), n.clone());

            lotus_rt::wait::next_tick().await;
        }
    });

    position
}

#[derive(Default, Clone, Copy)]
pub enum SollwertgeberMode {
    EmergencyBrake,
    Brake,
    #[default]
    Neutral,
    Throttle,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SollwertgeberNotch {
    EmergencyBrake,
    MaxBrake,
    Brake,
    MinBrake,
    Neutral,
    MinThrottle,
    Throttle,
    MaxTrottle,
}
