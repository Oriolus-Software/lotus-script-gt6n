use bon::Builder;
use lotus_rt::spawn;
use lotus_script::{time, var::VariableType};

use crate::standard_elements::Shared;

pub fn add_sollwertgeber(props: SollwertgeberProperties) -> Shared<f32> {
    let position = Shared::new(0.0);
    let speed = Shared::new(0.0);
    let target = Shared::new(0.0f32);
    let mode = Shared::new(SollwertgeberMode::Neutral);
    let notch = Shared::new(SollwertgeberNotch::Neutral);

    let p = position.clone();
    let sp = speed.clone();
    let md = mode.clone();
    let tg = target.clone();
    let props_clone = props.clone();
    spawn(async move {
        loop {
            lotus_rt::select! {
                _ = lotus_rt::wait::just_released(&props_clone.input_events.0)=>{}
                _ = lotus_rt::wait::just_released(&props_clone.input_events.1)=>{}
                _ = lotus_rt::wait::just_released(&props_clone.input_events.2)=>{}
            }
            let pos = p.get();
            let not_near_neutral = !(-0.1..0.1).contains(&pos);
            match md.get() {
                SollwertgeberMode::Throttle | SollwertgeberMode::Brake => {
                    if not_near_neutral {
                        tg.set_only_on_change(pos);
                        sp.set_only_on_change(0.0);
                    } else if tg.get() > 0.0 {
                        tg.set_only_on_change(0.1);
                    } else {
                        tg.set_only_on_change(-0.1);
                    }
                }
                _ => {}
            }
            props_clone.rw_lock.set_only_on_change(not_near_neutral);
        }
    });

    let p = position.clone();
    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let props_clone = props.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed(&props_clone.input_events.2).await;

            if !props_clone.lock.get() {
                let pos = p.get();
                let mode = md.get();

                match mode {
                    SollwertgeberMode::Throttle => {
                        if pos > 0.15 {
                            sp.set_only_on_change(-props_clone.speed.0);
                            tg.set_only_on_change(0.1);
                        } else {
                            sp.set_only_on_change(-props_clone.speed.1);
                            tg.set_only_on_change(0.0);
                            md.set_only_on_change(SollwertgeberMode::Neutral);
                        }
                    }
                    SollwertgeberMode::Neutral => {
                        sp.set_only_on_change(-props_clone.speed.0);
                        tg.set_only_on_change(-0.9);
                        md.set_only_on_change(SollwertgeberMode::Brake);
                    }
                    SollwertgeberMode::Brake => {
                        if pos > -0.9 {
                            sp.set_only_on_change(-props_clone.speed.0);
                            tg.set_only_on_change(-0.9);
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    let p = position.clone();
    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let props_clone = props.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed(&props_clone.input_events.0).await;

            if !props_clone.lock.get() {
                let pos = p.get();
                let mode = md.get();

                match mode {
                    SollwertgeberMode::Throttle => {
                        sp.set_only_on_change(props_clone.speed.0);
                        tg.set_only_on_change(1.0);
                    }
                    SollwertgeberMode::Neutral => {
                        sp.set_only_on_change(props_clone.speed.0);
                        tg.set_only_on_change(1.0);
                        md.set_only_on_change(SollwertgeberMode::Throttle);
                    }

                    SollwertgeberMode::Brake | SollwertgeberMode::EmergencyBrake => {
                        if pos < -0.15 {
                            sp.set_only_on_change(props_clone.speed.0);
                            tg.set_only_on_change(-0.1);
                            if pos < -0.9 {
                                p.set_only_on_change(-0.9);
                            }
                            md.set_only_on_change(SollwertgeberMode::Brake);
                        } else {
                            sp.set_only_on_change(props_clone.speed.1);
                            tg.set_only_on_change(0.0);
                            md.set_only_on_change(SollwertgeberMode::Neutral);
                        }
                    }
                }
            }
        }
    });

    let p = position.clone();
    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let props_clone = props.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed(&props_clone.input_events.1).await;

            if p.get() > 0.0 {
                sp.set_only_on_change(-props_clone.speed.1);
            } else {
                sp.set_only_on_change(props_clone.speed.1);
            }
            tg.set_only_on_change(0.0);
            md.set_only_on_change(SollwertgeberMode::Neutral);
        }
    });

    let sp = speed.clone();
    let tg = target.clone();
    let md = mode.clone();
    let props_clone = props.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed(&props_clone.input_events.3).await;

            if !props_clone.lock.get() {
                sp.set_only_on_change(-props_clone.speed.2);
                tg.set_only_on_change(-1.0);
                md.set_only_on_change(SollwertgeberMode::EmergencyBrake);
            }
        }
    });

    fn calc_sounds(
        pos: Shared<f32>,
        notch: Shared<SollwertgeberNotch>,
        props: SollwertgeberProperties,
    ) {
        let sound_neutral = props.sounds.0.clone();
        let sound_end = props.sounds.1.clone();
        let sound_other = props.sounds.2.clone();

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
                true.set(sound_neutral.as_str());
            } else if ((old_notch == SollwertgeberNotch::Throttle)
                && (new_notch == SollwertgeberNotch::MaxTrottle))
                || ((old_notch == SollwertgeberNotch::Brake)
                    && (new_notch == SollwertgeberNotch::MaxBrake))
            {
                true.set(sound_end.as_str());
            } else if !(((old_notch == SollwertgeberNotch::MaxTrottle)
                && (new_notch == SollwertgeberNotch::Throttle))
                || ((old_notch == SollwertgeberNotch::MaxBrake)
                    && (new_notch == SollwertgeberNotch::Brake)))
            {
                true.set(sound_other.as_str());
            }

            notch.set_only_on_change(new_notch);
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
                speed.set_only_on_change(0.0);
            }
            if (speed_val < 0.0) && (position < pos_min) {
                position = pos_min;
                speed.set_only_on_change(0.0);
            }

            p.set_only_on_change(position);
            position.set(&props.animation);

            calc_sounds(p.clone(), n.clone(), props.clone());

            lotus_rt::wait::next_tick().await;
        }
    });

    position
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum SollwertgeberMode {
    EmergencyBrake,
    Brake,
    #[default]
    Neutral,
    Throttle,
}

#[derive(PartialEq, Copy, Clone, Debug)]
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

#[derive(Clone, Builder)]
pub struct SollwertgeberProperties {
    #[builder(into)]
    pub animation: String,
    #[builder(into)]
    pub lock: Shared<bool>,
    #[builder(into)]
    pub rw_lock: Shared<bool>,
    /// (Throttle, Neutral, Brake, MaxBrake)
    #[builder(into)]
    pub input_events: (String, String, String, String),
    /// (Notch neutral, notch end, notch other
    #[builder(into)]
    pub sounds: (String, String, String),
    /// (Speed, Speed high, Speed very high)
    #[builder(into)]
    pub speed: (f32, f32, f32),
}
