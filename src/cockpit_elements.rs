use lotus_script::{action::state, delta, var::VariableType};

#[derive(Default)]
pub struct Cockpit {
    richtungswender: Richtungswender,
    sollwertgeber: Sollwertgeber,
}

impl Cockpit {
    pub fn tick(&mut self) {
        self.input();
        self.richtungswender.tick();
        self.sollwertgeber.tick();
        Tacho::calculate_value(f32::get("v_Axle_mps_0_1")).set("v_Axle_mps_0_1_abs");
    }

    fn input(&mut self) {
        if state("ReverserPlus").is_just_pressed() {
            self.richtungswender.plus();
        }
        if state("ReverserMinus").is_just_pressed() {
            self.richtungswender.minus();
        }
        if state("Throttle").is_pressed() {
            self.sollwertgeber.moving(1.0);
        }
        if state("Brake").is_pressed() {
            self.sollwertgeber.moving(-1.0);
        }
        if state("Neutral").is_pressed() {
            self.sollwertgeber.set(0.0);
        }
    }

    pub fn target_traction(&self) -> f32 {
        let traction_state = self.sollwertgeber.state.max(0.0);
        match self.richtungswender.state {
            RichtungswenderState::V => traction_state,
            RichtungswenderState::R => -traction_state,
            _ => 0.0,
        }
    }

    pub fn target_brake(&self) -> f32 {
        self.sollwertgeber.state.min(0.0)
    }
}

#[derive(Default)]
enum RichtungswenderState {
    #[default]
    O,
    I,
    V,
    R,
}

#[derive(Default)]
pub struct Richtungswender {
    state: RichtungswenderState,
}

impl Richtungswender {
    pub fn tick(&mut self) {
        self.angle().set("A_CP_Richtungswender");
    }

    pub fn plus(&mut self) {
        match self.state {
            RichtungswenderState::O => self.state = RichtungswenderState::I,
            RichtungswenderState::I => self.state = RichtungswenderState::V,
            RichtungswenderState::V => self.state = RichtungswenderState::R,
            _ => {}
        }
    }

    pub fn minus(&mut self) {
        match self.state {
            RichtungswenderState::I => self.state = RichtungswenderState::O,
            RichtungswenderState::V => self.state = RichtungswenderState::I,
            RichtungswenderState::R => self.state = RichtungswenderState::V,
            _ => {}
        }
    }

    pub fn angle(&self) -> f32 {
        match self.state {
            RichtungswenderState::I => 29.0,
            RichtungswenderState::V => 58.0,
            RichtungswenderState::R => 135.0,
            _ => 0.0,
        }
    }
}

#[derive(Default)]
pub struct Sollwertgeber {
    state: f32,
}

impl Sollwertgeber {
    fn tick(&mut self) {
        self.state.set("A_CP_Sollwertgeber");
    }

    fn moving(&mut self, speed: f32) {
        self.state = (self.state + speed * delta()).max(-1.0).min(1.0);
    }

    fn set(&mut self, position: f32) {
        self.state = position;
    }
}

#[derive(Default)]
pub struct Tacho;

impl Tacho {
    pub fn calculate_value(value: f32) -> f32 {
        value.abs()
    }
}
