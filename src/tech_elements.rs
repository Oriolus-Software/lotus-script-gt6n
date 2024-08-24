use lotus_script::{action, var::VariableType};

pub trait Tickable {
    fn tick(&mut self);
}

pub trait Inputable {
    fn input(&mut self);
}

pub struct Button {
    pressed: bool,
    input_event: String,
    animation_var: Option<String>,
    sound_on: Option<String>,
    sound_off: Option<String>,
}

impl Button {
    pub fn new(input_event: impl Into<String>) -> Self {
        Self {
            pressed: false,
            input_event: input_event.into(),
            animation_var: None,
            sound_on: None,
            sound_off: None,
        }
    }

    pub fn with_sound_on(mut self, sound_on: impl Into<String>) -> Self {
        self.sound_on = Some(sound_on.into());
        self
    }

    pub fn with_sound_off(mut self, sound_off: impl Into<String>) -> Self {
        self.sound_off = Some(sound_off.into());
        self
    }

    pub fn with_animation(mut self, animation_var: impl Into<String>) -> Self {
        self.animation_var = Some(animation_var.into());
        self
    }
}

impl Inputable for Button {
    fn input(&mut self) {
        if action::state(&self.input_event).kind.is_just_pressed() {
            self.pressed = true;

            if let Some(sound_on) = &self.sound_on {
                true.set(sound_on);
            }

            if let Some(anim_var) = &self.animation_var {
                1.0.set(anim_var);
            }
        }

        if action::state(&self.input_event).kind.is_just_released() {
            self.pressed = false;

            if let Some(sound_off) = &self.sound_off {
                true.set(sound_off)
            }

            if let Some(anim_var) = &self.animation_var {
                0.0.set(anim_var);
            }
        }
    }
}
