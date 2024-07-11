use lotus_script::{action::state, var::VariableType};

pub fn process_inputs() {
    state("Throttle")
        .is_pressed()
        .then_some(100_000.0)
        .unwrap_or_default()
        .set("M_Axle_N_0_0");
    state("Brake")
        .is_pressed()
        .then_some(100_000.0)
        .unwrap_or_default()
        .set("MBrake_Axle_N_0_0");
}
