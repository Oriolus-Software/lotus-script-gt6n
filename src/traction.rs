use lotus_script::var::VariableType;

use crate::cockpit_elements::Cockpit;

#[derive(Default)]
pub struct Traction {}

impl Traction {
    pub fn tick(&mut self, cockpit: &Cockpit) {
        (cockpit.target_traction() * 200_000.0).set("M_Axle_N_0_0");
        (cockpit.target_brake() * 300_000.0).set("MBrake_Axle_N_0_0");
    }
}
