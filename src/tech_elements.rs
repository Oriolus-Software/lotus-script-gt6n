use lotus_rt::{spawn, wait};
use lotus_script::var::VariableType;

pub fn add_button(prop: ButtonProperties) {
    spawn(async move {
        loop {
            wait::just_pressed(prop.input_event.clone().as_str()).await;
            if let Some(ref variable) = prop.animation_var {
                1.0.set(variable);
            }
            if let Some(ref sound) = prop.sound_on {
                true.set(sound);
            }
            wait::just_released(prop.input_event.clone().as_str()).await;
            if let Some(ref variable) = prop.animation_var {
                0.0.set(variable);
            }
            if let Some(ref sound) = prop.sound_off {
                true.set(sound);
            }
        }
    });
}

pub fn add_button_twosided_springloaded(prop: ButtonTwoSidedSpringLoaded) {
    fn set_on(target: f32, prop: &ButtonTwoSidedSpringLoaded) {
        if let Some(ref variable) = prop.animation_var {
            target.set(variable);
        }
        if let Some(ref sound) = prop.sound_on {
            true.set(sound);
        }
    }

    fn set_off(prop: &ButtonTwoSidedSpringLoaded) {
        if let Some(ref variable) = prop.animation_var {
            0.0.set(variable);
        }
        if let Some(ref sound) = prop.sound_off {
            true.set(sound);
        }
    }

    let p = prop.clone();
    spawn(async move {
        loop {
            wait::just_pressed(p.input_event_plus.clone().as_str()).await;
            set_on(1.0, &p);
            wait::just_released(p.input_event_plus.clone().as_str()).await;
            set_off(&p);
        }
    });

    spawn(async move {
        loop {
            wait::just_pressed(prop.input_event_minus.clone().as_str()).await;
            set_on(-1.0, &prop);
            wait::just_released(prop.input_event_minus.clone().as_str()).await;
            set_off(&prop);
        }
    });
}

pub struct ButtonProperties {
    pub input_event: String,
    pub animation_var: Option<String>,
    pub sound_on: Option<String>,
    pub sound_off: Option<String>,
}

#[derive(Clone)]
pub struct ButtonTwoSidedSpringLoaded {
    pub input_event_plus: String,
    pub input_event_minus: String,
    pub animation_var: Option<String>,
    pub sound_on: Option<String>,
    pub sound_off: Option<String>,
}
