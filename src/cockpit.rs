use lotus_rt::spawn;
use lotus_script::{time, var::VariableType};

#[derive(Default, Clone, Copy, PartialEq)]
pub enum RichtungswenderState {
    #[default]
    O,
    I,
    V,
    R,
}

pub fn add_richtungswender(
    lock: lotus_rt::sync::watch::Receiver<bool>,
) -> lotus_rt::sync::watch::Receiver<RichtungswenderState> {
    let (tx, rx) = lotus_rt::sync::watch::channel(RichtungswenderState::O);

    pub fn angle(state: RichtungswenderState) -> f32 {
        match state {
            RichtungswenderState::I => 29.0,
            RichtungswenderState::V => 58.0,
            RichtungswenderState::R => 135.0,
            _ => 0.0,
        }
    }

    let r = rx.clone();
    let t = tx.clone();

    let l = lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("ReverserPlus").await;

            if *l.borrow() {
                continue;
            }

            let mut state_new = None;

            match *r.borrow() {
                RichtungswenderState::O => state_new = Some(RichtungswenderState::I),
                RichtungswenderState::I => state_new = Some(RichtungswenderState::V),
                RichtungswenderState::V => state_new = Some(RichtungswenderState::R),
                _ => {}
            }

            if let Some(n) = state_new {
                t.send(n).unwrap();
                angle(n).set("A_CP_Richtungswender");
                true.set("Snd_CP_A_Reverser");
            }
        }
    });

    let r = rx.clone();
    let t = tx.clone();

    let l = lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("ReverserMinus").await;

            if *l.borrow() {
                continue;
            }

            let mut sn = None;

            match *r.borrow() {
                RichtungswenderState::I => sn = Some(RichtungswenderState::O),
                RichtungswenderState::V => sn = Some(RichtungswenderState::I),
                RichtungswenderState::R => sn = Some(RichtungswenderState::V),
                _ => {}
            }

            if let Some(n) = sn {
                t.send(n).unwrap();
                angle(n).set("A_CP_Richtungswender");
                true.set("Snd_CP_A_Reverser");
            }
        }
    });

    rx
}

pub fn add_sollwertgeber(
    richtungswender_r: lotus_rt::sync::watch::Receiver<RichtungswenderState>,
    rw_lock: lotus_rt::sync::watch::Sender<bool>,
) -> lotus_rt::sync::watch::Receiver<f32> {
    const SPEED: f32 = 1.0;
    const SPEED_HIGH: f32 = 5.0;
    const SPEED_VERYHIGH: f32 = 20.0;

    let (position_t, position_r) = lotus_rt::sync::watch::channel(0.0);
    let (speed_t, mut speed_r) = lotus_rt::sync::watch::channel(0.0);
    let (target_t, mut target_r) = lotus_rt::sync::watch::channel(0.0f32);
    let (mode_t, mode_r) = lotus_rt::sync::watch::channel(SollwertgeberMode::Neutral);
    let (notch_t, notch_r) = lotus_rt::sync::watch::channel(SollwertgeberNotch::Neutral);

    fn release(
        p_r: &lotus_rt::sync::watch::Receiver<f32>,
        sp_t: &lotus_rt::sync::watch::Sender<f32>,
        md_r: &lotus_rt::sync::watch::Receiver<SollwertgeberMode>,
        rw_lock: &lotus_rt::sync::watch::Sender<bool>,
        tg_t: &lotus_rt::sync::watch::Sender<f32>,
        tg_r: &lotus_rt::sync::watch::Receiver<f32>,
    ) {
        let pos = *p_r.borrow();
        let not_near_neutral = !(-0.1..0.1).contains(&pos);
        match *md_r.borrow() {
            SollwertgeberMode::Throttle | SollwertgeberMode::Brake => {
                if not_near_neutral {
                    tg_t.send(pos).unwrap();
                    sp_t.send(0.0).unwrap();
                } else if *tg_r.borrow() > 0.0 {
                    tg_t.send(0.1).unwrap();
                } else {
                    tg_t.send(-0.1).unwrap();
                }
            }
            _ => {}
        }
        rw_lock.send(not_near_neutral).unwrap();
    }

    let p_r = position_r.clone();
    let sp_t = speed_t.clone();
    let md_r = mode_r.clone();
    let tg_r = target_r.clone();
    let tg_t = target_t.clone();
    let rw_l = rw_lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_released("Throttle").await;
            release(&p_r, &sp_t, &md_r, &rw_l, &tg_t, &tg_r);
        }
    });

    let p_r = position_r.clone();
    let sp_t = speed_t.clone();
    let md_r = mode_r.clone();
    let tg_r = target_r.clone();
    let tg_t = target_t.clone();
    let rw_l = rw_lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_released("Brake").await;
            release(&p_r, &sp_t, &md_r, &rw_l, &tg_t, &tg_r);
        }
    });

    let p_r = position_r.clone();
    let sp_t = speed_t.clone();
    let md_r = mode_r.clone();
    let tg_r = target_r.clone();
    let tg_t = target_t.clone();
    let rw_l = rw_lock.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_released("Neutral").await;
            release(&p_r, &sp_t, &md_r, &rw_l, &tg_t, &tg_r);
        }
    });

    let p_r = position_r.clone();
    let sp_t = speed_t.clone();
    let tg_t = target_t.clone();
    let md_r = mode_r.clone();
    let md_t = mode_t.clone();
    let rw = richtungswender_r.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("Brake").await;

            match *rw.borrow() {
                RichtungswenderState::R | RichtungswenderState::V => {
                    let pos = *p_r.borrow();
                    let mode = *md_r.borrow();

                    match mode {
                        SollwertgeberMode::Throttle => {
                            if pos > 0.15 {
                                sp_t.send(-SPEED).unwrap();
                                tg_t.send(0.1).unwrap();
                            } else {
                                sp_t.send(-SPEED_HIGH).unwrap();
                                tg_t.send(0.0).unwrap();
                                md_t.send(SollwertgeberMode::Neutral).unwrap();
                            }
                        }
                        SollwertgeberMode::Neutral => {
                            sp_t.send(-SPEED).unwrap();
                            tg_t.send(-0.9).unwrap();
                            md_t.send(SollwertgeberMode::Brake).unwrap();
                        }
                        SollwertgeberMode::Brake => {
                            if pos > -0.9 {
                                sp_t.send(-SPEED).unwrap();
                                tg_t.send(-0.9).unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    let p_r = position_r.clone();
    let p_t = position_t.clone();
    let sp_t = speed_t.clone();
    let tg_t = target_t.clone();
    let md_r = mode_r.clone();
    let md_t = mode_t.clone();
    let rw = richtungswender_r.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("Throttle").await;

            match *rw.borrow() {
                RichtungswenderState::R | RichtungswenderState::V => {
                    let pos = *p_r.borrow();
                    let mode = *md_r.borrow();

                    match mode {
                        SollwertgeberMode::Throttle => {
                            sp_t.send(SPEED).unwrap();
                            tg_t.send(1.0).unwrap();
                        }
                        SollwertgeberMode::Neutral => {
                            sp_t.send(SPEED).unwrap();
                            tg_t.send(1.0).unwrap();
                            md_t.send(SollwertgeberMode::Throttle).unwrap();
                        }

                        SollwertgeberMode::Brake | SollwertgeberMode::EmergencyBrake => {
                            if pos < -0.15 {
                                sp_t.send(SPEED).unwrap();
                                tg_t.send(-0.1).unwrap();
                                if pos < -0.9 {
                                    p_t.send(-0.9).unwrap();
                                }
                                md_t.send(SollwertgeberMode::Brake).unwrap();
                            } else {
                                sp_t.send(SPEED_HIGH).unwrap();
                                tg_t.send(0.0).unwrap();
                                md_t.send(SollwertgeberMode::Neutral).unwrap();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    let p_r = position_r.clone();
    let sp_t = speed_t.clone();
    let tg_t = target_t.clone();
    let md_t = mode_t.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("Neutral").await;

            if *p_r.borrow() > 0.0 {
                sp_t.send(-SPEED_HIGH).unwrap();
            } else {
                sp_t.send(SPEED_HIGH).unwrap();
            }
            tg_t.send(0.0).unwrap();
            md_t.send(SollwertgeberMode::Neutral).unwrap();
        }
    });

    let sp_t = speed_t.clone();
    let tg_t = target_t.clone();
    let md_t = mode_t.clone();
    let rw = richtungswender_r.clone();
    spawn(async move {
        loop {
            lotus_rt::wait::just_pressed("MaxBrake").await;

            match *rw.borrow() {
                RichtungswenderState::R | RichtungswenderState::V => {
                    sp_t.send(-SPEED_VERYHIGH).unwrap();
                    tg_t.send(-1.0).unwrap();
                    md_t.send(SollwertgeberMode::EmergencyBrake).unwrap();
                }
                _ => {}
            }
        }
    });

    fn calc_sounds(
        pos_r: lotus_rt::sync::watch::Receiver<f32>,
        notch_t: lotus_rt::sync::watch::Sender<SollwertgeberNotch>,
        mut notch_r: lotus_rt::sync::watch::Receiver<SollwertgeberNotch>,
    ) {
        let pos = *pos_r.borrow();
        let old_notch = *notch_r.borrow_and_update();

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

            notch_t.send(new_notch).unwrap();
        }
    }

    let pr = position_r.clone();
    let pt = position_t.clone();
    let nr = notch_r.clone();
    let nt = notch_t.clone();
    spawn(async move {
        loop {
            let speed = *speed_r.borrow_and_update();
            let target = *target_r.borrow_and_update();

            // Das funktioniert nicht, solange die Position auch direkt gesetzt werden kann (z.B. per "Neutral")
            // if speed != 0.0 {
            let mut position = *pr.borrow() + speed * time::delta();

            let pos_max = target.min(1.0f32);
            let pos_min = target.max(-1.0f32);
            if (speed > 0.0) && (position > pos_max) {
                position = pos_max;
                speed_t.send(0.0).unwrap();
            }
            if (speed < 0.0) && (position < pos_min) {
                position = pos_min;
                speed_t.send(0.0).unwrap();
            }

            pt.send(position).unwrap();
            position.set("A_CP_Sollwertgeber");

            calc_sounds(pr.clone(), nt.clone(), nr.clone());

            lotus_rt::wait::next_tick().await;
        }
    });

    position_r
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
