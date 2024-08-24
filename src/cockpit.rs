use lotus_script::{
    action::state,
    delta,
    message::{BatterySwitch, MessageTarget},
    prelude::Message,
    var::VariableType,
};

use crate::tech_elements::{Button, Inputable};

pub struct Cockpit {
    richtungswender: Richtungswender,
    sollwertgeber: Sollwertgeber,
    tacho: Tacho,
    ts_lampentest: Button,
}

impl Cockpit {
    pub fn new() -> Self {
        Self {
            richtungswender: Richtungswender::default(),
            sollwertgeber: Sollwertgeber::default(),
            tacho: Tacho::new(
                "v_Axle_mps_0_1_abs".to_string(),
                "v_Axle_mps_0_1".to_string(),
            ),
            ts_lampentest: Button::new("Lightcheck")
                .with_sound_on("Snd_CP_A_BtnDn")
                .with_sound_off("Snd_CP_A_BtnUp")
                .with_animation("A_CP_TS_Lampentest"),
        }
    }

    pub fn tick(&mut self) {
        self.input();
        self.richtungswender.tick();
        self.sollwertgeber.tick();
        self.tacho.tick();
    }

    fn input(&mut self) {
        if state("ReverserPlus").kind.is_just_pressed() {
            self.richtungswender.plus();
            Message::new(BatterySwitch(true)).send(MessageTarget::Broadcast);
        }
        if state("ReverserMinus").kind.is_just_pressed() {
            self.richtungswender.minus();
            Message::new(BatterySwitch(false)).send(MessageTarget::Broadcast);
        }
        if state("Throttle").kind.is_pressed() {
            self.sollwertgeber.moving(1.0);
        }
        if state("Brake").kind.is_pressed() {
            self.sollwertgeber.moving(-1.0);
        }
        if state("Neutral").kind.is_pressed() {
            self.sollwertgeber.set(0.0);
        }

        self.ts_lampentest.input();
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
        (-self.sollwertgeber.state).max(0.0)
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
        true.set("Snd_Door_1_Open_Start");
    }

    pub fn minus(&mut self) {
        match self.state {
            RichtungswenderState::I => self.state = RichtungswenderState::O,
            RichtungswenderState::V => self.state = RichtungswenderState::I,
            RichtungswenderState::R => self.state = RichtungswenderState::V,
            _ => {}
        }
        false.set("Snd_Door_1_Open_Start");
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

pub struct Tacho {
    tacho_variable: String,
    axle_variable: String,
}

impl Tacho {
    pub fn new(tacho_var: impl Into<String>, axle_var: impl Into<String>) -> Self {
        Self {
            tacho_variable: tacho_var.into(),
            axle_variable: axle_var.into(),
        }
    }

    fn tick(&mut self) {
        (f32::get(&self.axle_variable).abs()).set(&self.tacho_variable);
    }
}
