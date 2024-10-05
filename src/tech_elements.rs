use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::{log, var::VariableType};

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

pub fn add_button_inout(prop: ButtonProperties) -> Shared<bool> {
    let pressed = Shared::<bool>::default();

    {
        let pressed = pressed.clone();

        spawn(async move {
            loop {
                let new_value = !pressed.get();

                wait::just_pressed(prop.input_event.clone().as_str()).await;

                pressed.set(new_value);

                log::info!("new value: {new_value}");

                if let Some(ref variable) = prop.animation_var {
                    2.0.set(variable);
                }
                if let Some(ref sound) = prop.sound_on {
                    true.set(sound);
                }

                wait::just_released(prop.input_event.clone().as_str()).await;

                if let Some(ref variable) = prop.animation_var {
                    (if new_value { 1.0 } else { 0.0 }).set(variable);
                }
                if let Some(ref sound) = prop.sound_off {
                    true.set(sound);
                }
            }
        });
    }

    pressed
}

pub fn add_step_switch(prop: StepSwitchProperties) -> Shared<i8> {
    fn set_position(prop: StepSwitchProperties, pos: &Shared<i8>, newval: i8) {
        if (prop.position_min..=prop.position_max).contains(&newval) {
            pos.set(newval);
            if let Some(sound) = prop.sound_move.clone() {
                true.set(sound.as_str());
            }
            if let Some(anim) = prop.animation_var.clone() {
                (newval as f32).set(anim.as_str());
            }
        }
    }

    let position = Shared::<i8>::default();

    {
        let prop = prop.clone();
        let position = position.clone();
        spawn(async move {
            loop {
                wait::just_pressed(prop.clone().input_event_plus.as_str()).await;

                set_position(prop.clone(), &position, position.get() + 1);
            }
        });
    }

    {
        let prop = prop.clone();
        let position = position.clone();
        spawn(async move {
            loop {
                wait::just_pressed(prop.clone().input_event_minus.as_str()).await;

                set_position(prop.clone(), &position, position.get() - 1);
            }
        });
    }

    position
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

                prop.voltage.switch(on).set(prop.variable.as_str());

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
pub struct StepSwitchProperties {
    #[builder(into)]
    pub position_max: i8,
    #[builder(into)]
    pub position_min: i8,
    #[builder(into)]
    pub input_event_plus: String,
    #[builder(into)]
    pub input_event_minus: String,
    #[builder(into)]
    pub animation_var: Option<String>,
    #[builder(into)]
    pub sound_move: Option<String>,
}

#[derive(Builder, Clone)]
pub struct IndicatorLightProperties {
    #[builder(into)]
    pub variable: String,
    pub lighttest: Option<Shared<bool>>,
    pub voltage: Shared<f32>,
}
