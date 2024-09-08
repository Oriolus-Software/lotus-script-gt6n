use lotus_rt::spawn;
use lotus_script::var::VariableType;

use crate::cockpit::RichtungswenderState;

const TRACTION_MULTIPLIER: f32 = 100_000.0;
const BRAKE_MULTIPLIER: f32 = 150_000.0;

pub fn add_traction(
    rx_sollwertgeber: lotus_rt::sync::watch::Receiver<f32>,
    rx_richtungswender: lotus_rt::sync::watch::Receiver<RichtungswenderState>,
) {
    let mut rx_wr = rx_richtungswender.clone();
    let mut rx_swg = rx_sollwertgeber.clone();

    spawn(async move {
        loop {
            rx_swg.changed().await.unwrap();

            let rw_position = *rx_wr.borrow_and_update();
            let swg_position = *rx_swg.borrow_and_update();
            let traction_state = swg_position.max(0.0);

            let target_traction = match rw_position {
                RichtungswenderState::V => traction_state,
                RichtungswenderState::R => -traction_state,
                _ => 0.0,
            };

            let target_brake = (-swg_position).max(0.0);

            (target_traction * TRACTION_MULTIPLIER).set("M_Axle_N_0_0");
            (target_brake * BRAKE_MULTIPLIER).set("MBrake_Axle_N_0_0");
        }
    });
}
