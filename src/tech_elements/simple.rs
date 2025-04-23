use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::{log, time, var::VariableType};

use crate::standard_elements::{exponential_approach, Shared};

pub fn add_copy<T: Copy + Default + 'static>(
    source: Shared<T>,
    existing_target: Option<&Shared<T>>,
) -> Shared<T> {
    let target = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<T>::default()
    };

    {
        let target = target.clone();

        source.on_change(
            move |v: &T| {
                target.set(*v);
            },
            "copy".to_string(),
        );
    }

    target
}

pub fn add_inverter(set: Shared<bool>, existing_target: Option<&Shared<bool>>) -> Shared<bool> {
    let inverted = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<bool>::default()
    };

    {
        let inverted = inverted.clone();

        set.on_change(
            move |v: &bool| {
                inverted.set(!v);
            },
            "inverter".to_string(),
        );
    }

    inverted
}

pub fn add_converter<I, O>(
    input: Shared<I>,
    f: impl Fn(&I) -> O + 'static,
    existing_target: Option<&Shared<O>>,
) -> Shared<O>
where
    I: std::fmt::Debug + Clone + PartialEq + 'static,
    O: Copy + Default + 'static,
{
    let output = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<O>::default()
    };

    {
        let output = output.clone();

        output.set(f(&input.get()));

        input.on_change(
            move |v: &I| {
                output.set(f(v));
            },
            "converter".to_string(),
        );
    }

    output
}

pub fn add_and(vec: Vec<Shared<bool>>, existing_target: Option<&Shared<bool>>) -> Shared<bool> {
    let anded = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<bool>::default()
    };

    {
        let anded = anded.clone();

        spawn(async move {
            loop {
                anded.set_only_on_change(vec.iter().all(|v| v.get()));

                wait::next_tick().await;
            }
        });
    }

    anded
}

pub fn add_or(vec: Vec<Shared<bool>>, existing_target: Option<&Shared<bool>>) -> Shared<bool> {
    let ored = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<bool>::default()
    };

    {
        let ored = ored.clone();

        spawn(async move {
            loop {
                ored.set_only_on_change(vec.iter().any(|v| v.get()));

                wait::next_tick().await;
            }
        });
    }

    ored
}

pub fn add_exponential_approach(
    prop: ExponentialApproachProperties,
    existing_target: Option<&Shared<f32>>,
) -> Shared<f32> {
    let state = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<f32>::default()
    };

    {
        let state = state.clone();

        spawn(async move {
            loop {
                state.set(exponential_approach(
                    state.get(),
                    prop.exponent,
                    prop.set_target.get(),
                ));
                wait::next_tick().await;
            }
        });
    }

    state
}

#[derive(Builder, Clone)]
pub struct ExponentialApproachProperties {
    /// always positive
    #[builder(into)]
    pub exponent: f32,
    #[builder(into)]
    pub set_target: Shared<f32>,
}

pub fn add_delay_relay(
    prop: DelayRelayProperties,
    existing_target: Option<&Shared<bool>>,
) -> Shared<bool> {
    let state = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<bool>::default()
    };

    {
        let state = state.clone();
        let mut set = prop.set.clone();
        let mut value = set.get();

        spawn(async move {
            loop {
                set.await_change().await;

                if set.get() != value {
                    value = set.get();

                    if value {
                        wait::seconds(prop.on_delay).await;
                    } else {
                        wait::seconds(prop.off_delay).await;
                    }

                    state.set(value);
                }
            }
        });
    }

    state
}

#[derive(Builder, Clone)]
pub struct DelayRelayProperties {
    #[builder(into)]
    pub on_delay: f32,
    #[builder(into)]
    pub off_delay: f32,
    #[builder(into)]
    pub set: Shared<bool>,
}

pub fn add_logger<T>(prop: Shared<T>, name: String)
where
    T: Clone + std::fmt::Debug + 'static,
{
    spawn(async move {
        loop {
            log::info!("{}: {:?}", name.clone(), prop.get());
            wait::next_tick().await;
        }
    });
}

pub fn add_timer(
    properties: TimerProperties,
    existing_target: Option<&Shared<bool>>,
) -> Shared<bool> {
    let state = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<bool>::default()
    };

    let time = properties.time;
    let set = properties.set;

    {
        let state = state.clone();

        spawn(async move {
            let mut timer = -0.0;

            loop {
                match set.get() {
                    TimerSet::Hold => timer = time,
                    TimerSet::LetRun => {
                        if timer > 0.0 {
                            timer -= time::delta();
                        }
                    }
                    TimerSet::Reset => timer = -0.1,
                }

                state.set_only_on_change(timer <= 0.0);

                wait::next_tick().await;
            }
        });
    }

    state
}

pub fn add_timer_var_time(
    properties: TimerVarTimeProperties,
    existing_target: Option<&Shared<bool>>,
) -> Shared<bool> {
    let state = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<bool>::default()
    };

    let time = properties.time;
    let set = properties.set;

    {
        let state = state.clone();

        spawn(async move {
            let mut timer: f32 = -0.0;

            loop {
                match set.get() {
                    TimerSetVarTime::Hold(Some(t)) => {
                        time.set_only_on_change(t);
                        timer = t;
                    }
                    TimerSetVarTime::Hold(None) => {
                        timer = time.get();
                    }
                    TimerSetVarTime::LetRun => {
                        if timer > 0.0 {
                            timer -= time::delta();
                        }
                    }
                    TimerSetVarTime::Reset => timer = -0.1,
                }

                timer = timer.min(time.get());

                state.set_only_on_change(timer <= 0.0);

                wait::next_tick().await;
            }
        });
    }

    state
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum TimerSet {
    #[default]
    Reset,
    Hold,
    LetRun,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum TimerSetVarTime {
    /// Reset the timer to 0.0 (expired = true)
    #[default]
    Reset,
    /// Hold the timer at the value "time" (expired = false)
    Hold(Option<f32>),
    /// Let the timer run (expired = false as long as the timer is greater than 0.0)
    LetRun,
}

#[derive(Builder, Clone)]
pub struct TimerProperties {
    #[builder(into)]
    pub time: f32,
    #[builder(into)]
    pub set: Shared<TimerSet>,
}

#[derive(Builder, Clone)]
pub struct TimerVarTimeProperties {
    #[builder(into)]
    pub time: Shared<f32>,
    #[builder(into)]
    pub set: Shared<TimerSetVarTime>,
}

pub fn add_blink_relais(
    prop: BlinkRelaisProperties,
    existing_target: Option<BlinkRelaisState>,
) -> BlinkRelaisState {
    let state = if let Some(target) = existing_target {
        target.clone()
    } else {
        BlinkRelaisState::default()
    };

    {
        let state = state.clone();

        spawn(async move {
            let reset_time = prop.reset_time.unwrap_or(0.0);
            let mut timer = reset_time;

            loop {
                let new_state = if prop.set_running.get() {
                    timer += time::delta();

                    if timer > prop.interval {
                        timer -= prop.interval;
                    }

                    timer < prop.on_time
                } else {
                    timer = reset_time;
                    false
                };

                state.on.set_only_on_change(new_state);

                wait::next_tick().await;
            }
        });
    }

    state
}

#[derive(Builder, Clone)]
pub struct BlinkRelaisProperties {
    #[builder(into)]
    pub interval: f32,
    #[builder(into)]
    pub on_time: f32,
    #[builder(into)]
    pub reset_time: Option<f32>,
    #[builder(into)]
    pub set_running: Shared<bool>,
}

#[derive(Clone, Default)]
pub struct BlinkRelaisState {
    pub on: Shared<bool>,
}

pub fn add_var_reader<T>(variable: String, existing_target: Option<&Shared<T>>) -> Shared<T>
where
    T: VariableType + Copy + PartialEq + Default + 'static,
    T::Output: Copy + PartialEq + Into<T>,
{
    let state = if let Some(target) = existing_target {
        target.clone()
    } else {
        Shared::<T>::default()
    };

    {
        let state = state.clone();

        let mut prev_value = T::get(&variable).into();
        state.set(prev_value);

        spawn(async move {
            loop {
                let new: T = T::get(&variable).into();
                if new != prev_value {
                    state.set(new);
                    prev_value = new;
                }
                wait::next_tick().await;
            }
        });
    }

    state
}

pub fn add_var_writer<T>(variable: String, value: Shared<T>)
where
    T: VariableType + 'static,
    T::Output: Into<T>,
{
    value.on_change(
        move |v: &T| {
            v.set(&variable);
        },
        "var_writer".to_string(),
    );
}

pub fn add_bool_to_float_var_unit(prop: BoolToFloatVarUnitProperties) {
    spawn(async move {
        let mut prev_bool = None;
        loop {
            let new_bool = prop.set_bool.get();
            if Some(new_bool) != prev_bool {
                let value = if new_bool { 1.0 } else { 0.0 };
                value.set(prop.float.as_str());
                prev_bool = Some(new_bool);
            }

            wait::next_tick().await;
        }
    });
}

#[derive(Builder, Clone)]
pub struct BoolToFloatVarUnitProperties {
    #[builder(into)]
    pub float: String,
    #[builder(into)]
    pub set_bool: Shared<bool>,
}

pub fn add_bool_to_sound_unit(prop: BoolToSoundUnitProperties) {
    spawn(async move {
        let mut prev_bool = false;
        loop {
            let new_bool = prop.set_bool.get();

            if new_bool != prev_bool {
                new_bool.set(prop.sound.as_str());
                prev_bool = new_bool;
            }

            wait::next_tick().await;
        }
    });
}
#[derive(Builder, Clone)]
pub struct BoolToSoundUnitProperties {
    #[builder(into)]
    pub sound: String,
    #[builder(into)]
    pub set_bool: Shared<bool>,
}

pub fn add_start_sound(properties: StartSoundProperties) {
    let previous_active = Shared::new(false);

    properties.set_active.on_change(
        move |active| {
            if previous_active.get() == *active {
                return;
            }

            if *active {
                if let Some(start_sound) = properties.start_sound.clone() {
                    true.set(start_sound.as_str());
                }
            }

            previous_active.set(*active);
        },
        "StartSound".to_string(),
    );
}

#[derive(Builder)]
pub struct StartSoundProperties {
    pub start_sound: Option<String>,
    pub set_active: Shared<bool>,
}

pub fn add_loop_sound(properties: LoopSoundProperties) {
    properties.set_active.on_change(
        move |active| {
            let a = *active;
            a.set(properties.loop_sound.as_str());
        },
        "LoopSound".to_string(),
    );
}

#[derive(Builder)]
pub struct LoopSoundProperties {
    pub loop_sound: String,
    pub set_active: Shared<bool>,
}
pub fn add_start_loop_stop_sound(properties: StartLoopStopSoundProperties) {
    let previous_active = Shared::new(false);

    properties.set_active.on_change(
        move |active| {
            if previous_active.get() == *active {
                return;
            }

            if *active {
                if let Some(start_sound) = properties.start_sound.clone() {
                    true.set(start_sound.as_str());
                }
                if let Some(loop_sound) = properties.loop_sound.clone() {
                    true.set(loop_sound.as_str());
                }
            } else {
                if let Some(loop_sound) = properties.loop_sound.clone() {
                    false.set(loop_sound.as_str());
                }
                if let Some(stop_sound) = properties.stop_sound.clone() {
                    true.set(stop_sound.as_str());
                }
            }

            previous_active.set(*active);
        },
        "StartLoopStopSound".to_string(),
    );
}

#[derive(Builder)]
pub struct StartLoopStopSoundProperties {
    pub start_sound: Option<String>,
    pub loop_sound: Option<String>,
    pub stop_sound: Option<String>,
    pub set_active: Shared<bool>,
}
