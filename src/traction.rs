use lotus_script::var::VariableType;

const TRACTION_MULTIPLIER: f32 = 100_000.0;
const BRAKE_MULTIPLIER: f32 = 150_000.0;

#[derive(Default)]
pub struct Traction {}

impl Traction {
    pub fn tick(&mut self, target_traction: f32, target_brake: f32) {
        (target_traction * TRACTION_MULTIPLIER).set("M_Axle_N_0_0");
        (target_brake * BRAKE_MULTIPLIER).set("MBrake_Axle_N_0_0");
    }
}
