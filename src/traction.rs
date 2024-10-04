use lotus_rt::{spawn, wait};
use lotus_script::var::VariableType;

use crate::standard_elements::{exponential_approach, Shared};

const MAXFORCE_N_ZERO: f32 = 16_000.0;
const MAXFORCE_N_60: f32 = 2000.0;
const MAXFORCE_DEC_PER_MS: f32 = (MAXFORCE_N_ZERO - MAXFORCE_N_60) / (60.0 / 3.6);
const VMAX: f32 = 60.0 / 3.6;
const VMAX_BACK: f32 = 15.0 / 3.6;
const V_EBRAKE_LIMIT: f32 = 5.0 / 3.6;
const MAXBRAKEFORCE_N: f32 = 16_000.0;

#[derive(Default)]
pub struct TractionAndBrakeUnitState {
    curr_traction_force: f32,
    curr_brake_force: f32,
    curr_speed: f32,
}

impl TractionAndBrakeUnitState {
    fn set_speed(&mut self, new_value: f32) {
        self.curr_speed = new_value;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TractionDirection {
    Forward,
    Neutral,
    Backward,
}

#[derive(Debug, Clone)]
pub struct ChannelsTraction {
    pub direction: Shared<TractionDirection>,
    pub target: Shared<f32>,
    pub federspeicher: Shared<bool>,
}

pub fn add_traction() -> ChannelsTraction {
    let direction = Shared::new(TractionDirection::Forward);
    let target = Shared::new(0.0);
    let federspeicher = Shared::new(false);

    let rx_wr = direction.clone();
    let rx_swg = target.clone();
    let fsp = federspeicher.clone();

    spawn(async move {
        let a = TractionAndBrakeUnitState::default();
        let b = TractionAndBrakeUnitState::default();
        let c = TractionAndBrakeUnitState::default();
        let mut units = [a, b, c];

        loop {
            let rw_position = rx_wr.get();
            let swg_position = rx_swg.get();

            let fast_brake = swg_position < -0.95;
            let emergency_brake = false;

            let max_e_brake = fast_brake || emergency_brake;

            let schleuderschutz_active = false;
            let gleitschutz_active = false;

            let traction_power_avl = true;

            units
                .get_mut(0)
                .unwrap()
                .set_speed(f32::get("v_Axle_mps_0_1"));
            units
                .get_mut(1)
                .unwrap()
                .set_speed(f32::get("v_Axle_mps_1_1"));
            units
                .get_mut(2)
                .unwrap()
                .set_speed(f32::get("v_Axle_mps_2_0"));

            for u in units.iter_mut() {
                let reversed = rw_position == TractionDirection::Backward;

                let speed_in_dir = if reversed {
                    -u.curr_speed
                } else {
                    u.curr_speed
                };

                let max_wheelforce = MAXFORCE_N_ZERO - u.curr_speed.abs() * MAXFORCE_DEC_PER_MS;

                let target_all_brakes = if max_e_brake {
                    -1.0
                } else if swg_position < 0.0 {
                    swg_position * 1.111
                } else if (!reversed && speed_in_dir > VMAX)
                    || (reversed && speed_in_dir > VMAX_BACK)
                    || schleuderschutz_active
                {
                    0.0
                } else {
                    swg_position
                };

                let mut target_e_brake = target_all_brakes;

                if target_all_brakes >= 0.0 {
                    0.0.set("Snd_BrakeFlirr");
                    target_e_brake *= max_wheelforce * if reversed { -1.0 } else { 1.0 };
                } else {
                    target_e_brake =
                        target_e_brake.max(-1.0) * (u.curr_speed.abs() / V_EBRAKE_LIMIT).min(1.0);
                    if u.curr_speed < 0.0 {
                        target_e_brake = -target_e_brake;
                    }
                    if gleitschutz_active {
                        target_e_brake /= 3.0;
                    }
                    target_e_brake.abs().set("Snd_BrakeFlirr");
                    target_e_brake *= MAXBRAKEFORCE_N;
                };

                if traction_power_avl {
                    u.curr_traction_force =
                        exponential_approach(u.curr_traction_force, 10.0, target_e_brake);
                } else {
                    u.curr_traction_force = 0.0;
                }

                let mut target_pneu_brake = if target_all_brakes >= 0.0 {
                    if speed_in_dir < 0.0 {
                        1.0
                    } else if (speed_in_dir == 0.0)
                        && ((target_all_brakes == 0.0)
                            || (u.curr_traction_force.abs() < 0.8 * target_e_brake.abs()))
                    {
                        1.0
                    } else {
                        0.0
                    }
                } else if target_all_brakes > -1.1 {
                    1.0 - (u.curr_speed.abs() / V_EBRAKE_LIMIT).min(1.0)
                } else {
                    1.0
                };

                if gleitschutz_active {
                    target_pneu_brake /= 3.0;
                }

                // Federspeicherbremse fehlt:
                u.curr_brake_force =
                    (target_pneu_brake.max(fsp.get() as u8 as f32)) * MAXBRAKEFORCE_N;
            }

            units[0].curr_traction_force.set("M_Axle_N_0_1");
            units[1].curr_traction_force.set("M_Axle_N_1_1");
            units[2].curr_traction_force.set("M_Axle_N_2_0");

            units[0].curr_brake_force.set("MBrake_Axle_N_0_1");
            units[1].curr_brake_force.set("MBrake_Axle_N_1_1");
            units[2].curr_brake_force.set("MBrake_Axle_N_2_0");

            wait::next_tick().await;
        }
    });

    ChannelsTraction {
        direction,
        target,
        federspeicher,
    }
}
