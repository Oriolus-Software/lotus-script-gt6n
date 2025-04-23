use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::{log, var::VariableType};

use crate::standard_elements::{multiple_on_change, Shared};

pub async fn just_pressed_check_cockit_index(id: &str, index: usize) {
    loop {
        let pressed = wait::just_pressed(id).await;
        log::info!("pressed: {:?}", pressed);
        match pressed.cockpit_index {
            Some(cockpit_index) => {
                if cockpit_index == index {
                    break;
                }
            }
            None => {
                break;
            }
        }
    }
}

pub async fn just_released_check_cockit_index(id: &str, index: usize) {
    loop {
        let released = wait::just_released(id).await;
        log::info!("released: {:?}", released);
        match released.cockpit_index {
            Some(cockpit_index) => {
                if cockpit_index == index {
                    break;
                }
            }
            None => {
                break;
            }
        }
    }
}

pub fn add_button(prop: ButtonProperties) -> Shared<bool> {
    let pressed = Shared::<bool>::default();

    {
        let pressed = pressed.clone();

        spawn(async move {
            loop {
                just_pressed_check_cockit_index(prop.input_event.as_str(), 0).await;

                pressed.set(true);
                if let Some(ref variable) = prop.animation_var {
                    1.0.set(variable);
                }
                if let Some(ref sound) = prop.sound_on {
                    true.set(sound);
                }

                just_released_check_cockit_index(prop.input_event.as_str(), 0).await;

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

/// Typical modern touch button: After touching it, it stays "pressed" for a short time. You cannot activate it again until the timer has run out.
pub fn add_timed_button(prop: TimedButtonProperties) -> Shared<bool> {
    let pressed = Shared::<bool>::default();

    {
        let pressed = pressed.clone();

        spawn(async move {
            loop {
                wait::just_pressed(prop.input_event.clone().as_str()).await;
                pressed.set(true);
                wait::seconds(prop.time_staying_on).await;
                pressed.set(false);
                wait::seconds(prop.time_before_pressable_again).await;
            }
        });
    }

    pressed
}

pub fn add_button_twosided_springloaded(
    prop: ButtonTwoSidedSpringLoadedProperties,
) -> Shared<ButtonTwoSidedSpringLoadedState> {
    let pressed = Shared::<ButtonTwoSidedSpringLoadedState>::default();

    fn set_on(
        target: f32,
        prop: &ButtonTwoSidedSpringLoadedProperties,
        pressed: Shared<ButtonTwoSidedSpringLoadedState>,
    ) {
        if target > 0.0 {
            pressed.set(ButtonTwoSidedSpringLoadedState::HoldOn);
        } else {
            pressed.set(ButtonTwoSidedSpringLoadedState::HoldOff);
        }
        if let Some(ref variable) = prop.animation_var {
            target.set(variable);
        }
        if let Some(ref sound) = prop.sound_on {
            true.set(sound);
        }
    }

    fn set_off(
        prop: &ButtonTwoSidedSpringLoadedProperties,
        pressed: Shared<ButtonTwoSidedSpringLoadedState>,
    ) {
        pressed.set(ButtonTwoSidedSpringLoadedState::Released);
        if let Some(ref variable) = prop.animation_var {
            0.0.set(variable);
        }
        if let Some(ref sound) = prop.sound_off {
            true.set(sound);
        }
    }
    {
        let pressed = pressed.clone();
        let prop = prop.clone();
        spawn(async move {
            loop {
                wait::just_pressed(prop.input_event_plus.clone().as_str()).await;
                set_on(1.0, &prop, pressed.clone());
                wait::just_released(prop.input_event_plus.clone().as_str()).await;
                set_off(&prop, pressed.clone());
            }
        });
    }
    {
        let pressed = pressed.clone();
        let prop = prop.clone();
        spawn(async move {
            loop {
                wait::just_pressed(prop.input_event_minus.clone().as_str()).await;
                set_on(-1.0, &prop, pressed.clone());
                wait::just_released(prop.input_event_minus.clone().as_str()).await;
                set_off(&prop, pressed.clone());
            }
        });
    }

    pressed
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

pub fn add_switch(prop: SwitchProperties) -> Shared<bool> {
    let position = Shared::<bool>::default();

    {
        let position = position.clone();
        let prop = prop.clone();

        spawn(async move {
            let set_position = |newval: bool| {
                position.set(newval);
                if let Some(animation_var) = prop.animation_var.clone() {
                    (if newval { 1.0 } else { 0.0 }).set(animation_var.as_str());
                }
            };

            if let Some(standard_position) = prop.standard_position {
                set_position(standard_position);
            }

            loop {
                wait::just_pressed(prop.input_event.clone().as_str()).await;
                set_position(!position.get());
                if let Some(sound) = prop.sound_switch.clone() {
                    true.set(sound.as_str());
                }
            }
        });
    }

    position
}
pub fn add_step_switch(prop: StepSwitchProperties) -> Shared<i8> {
    let position = Shared::<i8>::default();

    fn set_position(
        prop: &StepSwitchProperties,
        position: &Shared<i8>,
        newval: i8,
        play_sound: bool,
    ) {
        if (prop.position_min..=prop.position_max).contains(&newval) {
            position.set(newval);
            if let Some(sound) = &prop.sound_switch {
                if play_sound {
                    true.set(sound.as_str());
                }
            }

            if let Some(anim) = &prop.animation_var {
                (newval as f32).set(anim.as_str());
            }
        }
    }

    if let Some(standard_position) = prop.standard_position {
        set_position(&prop, &position, standard_position, false);
    }
    {
        let position = position.clone();
        let prop = prop.clone();

        spawn(async move {
            loop {
                wait::just_pressed(prop.input_event_plus.as_str()).await;
                set_position(&prop, &position, position.get() + 1, true);
            }
        });
    }
    {
        let position = position.clone();
        let prop = prop.clone();

        spawn(async move {
            loop {
                wait::just_pressed(prop.input_event_minus.as_str()).await;

                set_position(&prop, &position, position.get() - 1, true);

                if prop.position_min_is_springloaded.unwrap_or(false)
                    && position.get() == prop.position_min
                {
                    wait::just_released(prop.input_event_minus.as_str()).await;
                    set_position(&prop, &position, prop.position_min + 1, true);
                }
            }
        });
    }

    position
}

pub fn add_complex_step_switch<
    T: ComplexStepSwitchState + Into<i8> + From<i8> + Default + Clone + Copy + 'static,
>(
    prop: ComplexStepSwitchProperties,
) -> Shared<T> {
    let state = Shared::<T>::default();

    let process = |input_event: String, step: i8| {
        let state = state.clone();
        let prop = prop.clone();

        if let Some(anim) = &prop.animation_var {
            state.get().get_angle().set(anim.as_str());
        }

        spawn(async move {
            loop {
                wait::just_pressed(input_event.as_str()).await;

                if prop.blocked.clone().map(|b| b.get()).unwrap_or(false) {
                    continue;
                }

                let new_state = state.get().into() + step;

                if (new_state > T::max()) || (new_state < 0) {
                    continue;
                }

                let new_state = new_state.into();
                state.set(new_state);

                if let Some(anim) = &prop.animation_var {
                    new_state.get_angle().set(anim.as_str());
                }

                if let Some(sound) = &prop.sound_switch {
                    true.set(sound.as_str());
                }
            }
        });
    };

    process(prop.input_event_plus.clone(), 1);
    process(prop.input_event_minus.clone(), -1);

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
pub struct TimedButtonProperties {
    #[builder(into)]
    pub input_event: String,
    #[builder(into)]
    pub time_staying_on: f32,
    #[builder(into)]
    pub time_before_pressable_again: f32,
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

#[derive(Debug, Clone, Default)]
pub enum ButtonTwoSidedSpringLoadedState {
    HoldOn,
    #[default]
    Released,
    HoldOff,
}

#[derive(Builder, Clone)]
pub struct SwitchProperties {
    #[builder(into)]
    pub input_event: String,
    #[builder(into)]
    pub animation_var: Option<String>,
    #[builder(into)]
    pub sound_switch: Option<String>,
    #[builder(into)]
    pub standard_position: Option<bool>,
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
    pub sound_switch: Option<String>,
    #[builder(into)]
    pub standard_position: Option<i8>,
    #[builder(into)]
    pub position_min_is_springloaded: Option<bool>,
}

pub trait ComplexStepSwitchState {
    fn get_angle(&self) -> f32;
    fn max() -> i8;
}

#[derive(Builder, Clone)]
pub struct ComplexStepSwitchProperties {
    #[builder(into)]
    pub input_event_plus: String,
    #[builder(into)]
    pub input_event_minus: String,
    #[builder(into)]
    pub animation_var: Option<String>,
    #[builder(into)]
    pub sound_switch: Option<String>,
    #[builder(into)]
    pub standard_position: Option<i8>,
    #[builder(into)]
    pub blocked: Option<Shared<bool>>,
}

pub fn add_indicator_light(prop: IndicatorLightProperties) -> Shared<bool> {
    let state = Shared::<bool>::default();

    {
        let state = state.clone();
        0.0.set(prop.variable.as_str());

        multiple_on_change(
            &[
                &prop.lighttest.clone(),
                &prop.voltage.clone(),
                &state.clone(),
            ],
            move || {
                let mut on = state.get();
                if let Some(lt) = &prop.lighttest {
                    on |= lt.get();
                }
                prop.voltage.switch(on).set(&prop.variable);
            },
        );
    }

    state
}

#[derive(Builder, Clone)]
pub struct IndicatorLightProperties {
    #[builder(into)]
    pub variable: String,
    pub lighttest: Option<Shared<bool>>,
    pub voltage: Shared<f32>,
}
