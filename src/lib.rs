use input::process_inputs;
use lotus_script::{action::RegisterAction, delta_f64, script, var::VariableType, Script};

pub mod input;

script!(ScriptGt6n);

#[derive(Default)]
pub struct ScriptGt6n {
    delta: f64,
}

impl Script for ScriptGt6n {
    fn init(&mut self) {}

    fn actions() -> Vec<RegisterAction> {
        Vec::new()
    }

    fn tick(&mut self) {
        self.delta += delta_f64();

        process_inputs();

        ((self.delta * 100.0).sin() > 0.0)
            .then_some(1.0)
            .unwrap_or_default()
            .set("A_LM_Fernlicht");
    }
}
