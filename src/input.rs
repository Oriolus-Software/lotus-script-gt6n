use lotus_script::{action::state, var::set_var};

pub fn process_inputs() {
    set_var(
        "M_Axle_N_0_0",
        &state("Throttle")
            .kind
            .is_pressed()
            .then_some(100_000.0)
            .unwrap_or_default(),
    );
    set_var(
        "MBrake_Axle_N_0_0",
        &state("Brake")
            .kind
            .is_pressed()
            .then_some(100_000.0)
            .unwrap_or_default(),
    );
}
