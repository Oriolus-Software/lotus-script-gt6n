use cockpit_elements::Cockpit;
use input::process_inputs;
use lotus_script::{action::RegisterAction, script, Script};
use traction::Traction;

pub mod cockpit_elements;
pub mod input;
pub mod traction;

script!(ScriptGt6n);

pub struct ScriptGt6n {
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
    }
}

impl Default for ScriptGt6n {
    fn default() -> Self {
        Self {
            cockpit: Cockpit::new(),
            traction: Traction::default(),
        }
    }
}
