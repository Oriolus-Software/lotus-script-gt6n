use cockpit_elements::Cockpit;
use input::process_inputs;
use lotus_script::{action::RegisterAction, delta, script, var::VariableType, Script};
use traction::Traction;

pub mod cockpit_elements;
pub mod input;
pub mod traction;

script!(ScriptGt6n);

pub struct ScriptGt6n {
    timer: f32,
    cockpit: Cockpit,
    traction: Traction,
}

impl Script for ScriptGt6n {
    fn init(&mut self) {}

    fn actions() -> Vec<RegisterAction> {
        Vec::new()
    }

    fn tick(&mut self) {
        process_inputs();

        self.cockpit.tick();
        self.traction.tick(&self.cockpit);

        self.timer += delta();
        ((self.timer).sin() * 0.5 + 0.5).set("A_LM_Fernlicht");
    }
}

impl Default for ScriptGt6n {
    fn default() -> Self {
        Self {
            cockpit: Cockpit::new(),
            traction: Traction::default(),
            timer: 0.0,
        }
    }
}
