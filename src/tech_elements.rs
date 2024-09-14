use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::var::VariableType;

use crate::standard_elements::Shared;

pub fn add_button(prop: ButtonProperties) -> Shared<bool> {
    let pressed = Shared::<bool>::default();

    {
        let pressed = pressed.clone();

        spawn(async move {
            loop {
                wait::just_pressed(prop.input_event.clone().as_str()).await;

                pressed.set(true);
                if let Some(ref variable) = prop.animation_var {
                    1.0.set(variable);
                }
                if let Some(ref sound) = prop.sound_on {
                    true.set(sound);
                }

                wait::just_released(prop.input_event.clone().as_str()).await;

                pressed.set(false);
                if let Some(ref variable) = prop.animation_var {
                    0.0.set(variable);
                }
                if let Some(ref sound) = prop.sound_off {
                    true.set(sound);
                }
            }
        });
    }

    pressed
}

pub fn add_button_twosided_springloaded(prop: ButtonTwoSidedSpringLoadedProperties) {
    fn set_on(target: f32, prop: &ButtonTwoSidedSpringLoadedProperties) {
        if let Some(ref variable) = prop.animation_var {
            target.set(variable);
        }
        if let Some(ref sound) = prop.sound_on {
            true.set(sound);
        }
    }

    fn set_off(prop: &ButtonTwoSidedSpringLoadedProperties) {
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

pub fn add_indicator_light(prop: IndicatorLightProperties) -> Shared<bool> {
    let state = Shared::<bool>::default();

    {
        let state = state.clone();

        spawn(async move {
            loop {
                let mut on = state.get();
                if let Some(ref lt) = prop.lighttest {
                    on = on || lt.get();
                }

                if on { prop.voltage.get() } else { 0.0 }.set(prop.variable.as_str());

                wait::next_tick().await;
            }
        });
    }

    state
}

#[derive(Builder, Clone)]
pub struct ButtonProperties {
    #[builder(into)]
    pub input_event: String,
    #[builder(into)]
    pub animation_var: Option<String>,
    #[builder(into)]
    pub sound_on: Option<String>,
    #[builder(into)]
    pub sound_off: Option<String>,
}

#[derive(Builder, Clone)]
pub struct ButtonTwoSidedSpringLoadedProperties {
    #[builder(into)]
    pub input_event_plus: String,
    #[builder(into)]
    pub input_event_minus: String,
    #[builder(into)]
    pub animation_var: Option<String>,
    #[builder(into)]
    pub sound_on: Option<String>,
    #[builder(into)]
    pub sound_off: Option<String>,
}

#[derive(Builder, Clone)]
pub struct IndicatorLightProperties {
    #[builder(into)]
    pub variable: String,
    pub lighttest: Option<Shared<bool>>,
    pub voltage: Shared<f32>,
}
