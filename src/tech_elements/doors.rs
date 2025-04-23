use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::{time, var::VariableType};

use crate::standard_elements::Shared;

use super::simple::{
    add_inverter, add_timer, add_timer_var_time, TimerProperties, TimerSet, TimerSetVarTime,
    TimerVarTimeProperties,
};

pub fn add_electric_sliding_plug_door_pair(
    prop: ElectricSlidingPlugDoorPairProperties,
) -> ElectricSlidingPlugDoorPairState {
    let state = ElectricSlidingPlugDoorPairState::default();

    {
        let state = state.clone();

        spawn(async move {
            let mut position = 0.0;
            let mut speed = 0.0;

            loop {
                let target = state.set_target.get();

                if speed != 0.0
                    || (target == ElectricSlidingPlugDoorPairTarget::Open && position < 1.0)
                    || (target == ElectricSlidingPlugDoorPairTarget::Close && position > 0.0)
                {
                    let acc: f32;
                    let v_soll: f32;

                    if target == ElectricSlidingPlugDoorPairTarget::NoEnergy {
                        acc = if speed > 0.0 {
                            -prop.friction
                        } else if speed < 0.0 {
                            prop.friction
                        } else {
                            0.0
                        };
                    } else {
                        if target == ElectricSlidingPlugDoorPairTarget::Open {
                            if let Some(ref sound) = prop.sound_open_start {
                                if position < 0.01 && speed <= 0.0 {
                                    true.set(sound.as_str());
                                }
                            }
                            if position < prop.open_start_end_change_position {
                                v_soll = prop.open_start_speed;
                            } else {
                                v_soll = prop.open_end_speed;
                            }
                        } else {
                            if let Some(ref sound) = prop.sound_close_start {
                                if position > 0.1 && speed >= 0.0 {
                                    true.set(sound.as_str());
                                }
                            }
                            if position > prop.close_start_end_change_position {
                                v_soll = -prop.close_start_speed;
                            } else {
                                v_soll = -prop.close_end_speed;
                            }
                        }
                        acc = (v_soll - speed) * prop.traction_stiftness;
                    }

                    let new_speed = speed + acc * time::delta();

                    if new_speed * speed < 0.0 {
                        speed = 0.0;
                    } else {
                        speed = new_speed;
                    }

                    let mut new_position = position + speed * time::delta();

                    if let Some(ref sound) = prop.sound_close_transition {
                        if new_position < 0.1 && position >= 0.1 && speed < 0.0 {
                            true.set(sound.as_str());
                        }
                    }
                    if let Some(ref sound) = prop.sound_open_transition {
                        if new_position > 0.1 && position <= 0.1 && speed > 0.0 {
                            true.set(sound.as_str());
                        }
                    }
                    if let Some(ref sound) = prop.sound_close_end {
                        if new_position < 0.01 && position >= 0.01 {
                            true.set(sound.as_str());
                        }
                    }

                    if new_position > 1.0 {
                        new_position = 1.0;
                        speed = -speed * prop.reflection_open;
                        if let Some(ref sound) = prop.sound_open_end {
                            true.set(sound.as_str());
                        }
                    } else if new_position < 0.0 {
                        new_position = 0.0;
                        speed = -speed * prop.reflection_close;
                    }

                    position = new_position;
                }

                let x: f32;
                let shift_position: f32;

                if position < 0.1 {
                    let arc = position * 5.0 * std::f32::consts::PI;
                    x = prop.plug_radius * arc.sin();
                    shift_position = prop.plug_radius * (1.0 - arc.cos());
                } else {
                    x = prop.plug_radius;
                    shift_position =
                        (position - 0.1) / 0.9 * prop.shift_distance + prop.plug_radius;
                }

                state.y_position_blade_a.set_only_on_change(shift_position);
                state.y_position_blade_b.set_only_on_change(-shift_position);
                state.x_position_rail.set_only_on_change(x);

                if let Some(ref variable_x_rail) = prop.variable_x_rail {
                    x.set(variable_x_rail.as_str());
                }
                if let Some(ref variable_y_blade_a) = prop.variable_y_blade_a {
                    shift_position.set(variable_y_blade_a.as_str());
                }
                if let Some(ref variable_y_blade_b) = prop.variable_y_blade_b {
                    (-shift_position).set(variable_y_blade_b.as_str());
                }

                state.position.set_only_on_change(if position < 0.01 {
                    ElectricSlidingPlugDoorPairPositionState::FullyClosed
                } else if position > 0.99 {
                    ElectricSlidingPlugDoorPairPositionState::FullyOpen
                } else {
                    ElectricSlidingPlugDoorPairPositionState::InTransition
                });

                wait::next_tick().await;
            }
        });
    }

    state
}

#[derive(Builder, Clone)]
pub struct ElectricSlidingPlugDoorPairProperties {
    #[builder(into)]
    pub plug_radius: f32,
    #[builder(into)]
    pub shift_distance: f32,

    #[builder(into)]
    pub friction: f32,

    #[builder(into)]
    pub open_start_speed: f32,
    #[builder(into)]
    pub open_end_speed: f32,
    #[builder(into)]
    pub open_start_end_change_position: f32,

    #[builder(into)]
    pub close_start_speed: f32,
    #[builder(into)]
    pub close_end_speed: f32,
    #[builder(into)]
    pub close_start_end_change_position: f32,

    #[builder(into)]
    pub traction_stiftness: f32,
    #[builder(into)]
    pub reflection_open: f32,
    #[builder(into)]
    pub reflection_close: f32,

    #[builder(into)]
    pub sound_open_start: Option<String>,
    #[builder(into)]
    pub sound_open_transition: Option<String>,
    #[builder(into)]
    pub sound_open_end: Option<String>,
    #[builder(into)]
    pub sound_close_start: Option<String>,
    #[builder(into)]
    pub sound_close_transition: Option<String>,
    #[builder(into)]
    pub sound_close_end: Option<String>,

    #[builder(into)]
    pub variable_x_rail: Option<String>,
    #[builder(into)]
    pub variable_y_blade_a: Option<String>,
    #[builder(into)]
    pub variable_y_blade_b: Option<String>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum ElectricSlidingPlugDoorPairTarget {
    #[default]
    NoEnergy,
    Open,
    Close,
}

#[derive(Clone, Default, Eq, PartialEq, Debug, Copy)]
pub enum ElectricSlidingPlugDoorPairPositionState {
    #[default]
    InTransition,
    FullyOpen,
    FullyClosed,
}

#[derive(Clone, Default, Debug)]
pub struct ElectricSlidingPlugDoorPairState {
    pub y_position_blade_a: Shared<f32>,
    pub y_position_blade_b: Shared<f32>,
    pub x_position_rail: Shared<f32>,
    pub position: Shared<ElectricSlidingPlugDoorPairPositionState>,
    pub set_target: Shared<ElectricSlidingPlugDoorPairTarget>,
}

//-------------------------------------------

pub fn add_door_control(prop: DoorControlProperties) -> DoorControlState {
    let state = DoorControlState::default();

    let open_timer_set = Shared::<TimerSetVarTime>::default();
    let open_timer_time = Shared::<f32>::default();

    spawn(door_control_set_open_timer(
        prop.clone(),
        open_timer_set.clone(),
        open_timer_time.clone(),
    ));

    let open_timer = add_timer_var_time(
        TimerVarTimeProperties::builder()
            .time(open_timer_time)
            .set(open_timer_set.clone())
            .build(),
        None,
    );

    {
        let set_released = prop.set_released.clone();
        let set_door_closed = prop.set_door_closed.clone();

        prop.set_released.on_change(
            move |_| {
                if set_released.get()
                    && set_door_closed.get()
                        != ElectricSlidingPlugDoorPairPositionState::FullyClosed
                {
                    open_timer_set.set(TimerSetVarTime::Hold(Some(prop.request_time)));
                }
            },
            "Door open on release".to_string(),
        );
    }

    {
        let warning = state.warning.clone();
        let set_released = prop.set_released.clone();
        let set_door_closed = prop.set_door_closed.clone();

        spawn(async move {
            loop {
                warning.set_only_on_change(
                    !set_released.get()
                        && set_door_closed.get()
                            != ElectricSlidingPlugDoorPairPositionState::FullyClosed,
                );
                wait::next_tick().await;
            }
        });
    }

    spawn(door_control_set_door_target(
        prop.set_system_active,
        open_timer,
        prop.set_force,
        prop.get_door_target,
    ));

    state
}

async fn door_control_set_open_timer(
    prop: DoorControlProperties,
    timer_set: Shared<TimerSetVarTime>,
    timer_time: Shared<f32>,
) {
    loop {
        if prop.set_released.get() {
            timer_time.set_only_on_change(prop.request_time);
        } else {
            timer_time.set_only_on_change(prop.warning_time);
        }

        if !prop.set_system_active.get() {
            timer_set.set_only_on_change(TimerSetVarTime::Reset);
        } else if prop.set_request.get() {
            timer_set.set_only_on_change(TimerSetVarTime::Hold(None));
        } else {
            timer_set.set_only_on_change(TimerSetVarTime::LetRun);
        }

        wait::next_tick().await;
    }
}

async fn door_control_set_door_target(
    system_active: Shared<bool>,
    open_timer_expired: Shared<bool>,
    force_option: Option<Shared<DoorControlMode>>,
    door_target: Shared<ElectricSlidingPlugDoorPairTarget>,
) {
    loop {
        let force = if let Some(force) = &force_option {
            force.get()
        } else {
            DoorControlMode::Automatic
        };

        door_target.set_only_on_change(if system_active.get() {
            match force {
                DoorControlMode::Automatic => {
                    if open_timer_expired.get() {
                        ElectricSlidingPlugDoorPairTarget::Close
                    } else {
                        ElectricSlidingPlugDoorPairTarget::Open
                    }
                }
                DoorControlMode::Open => ElectricSlidingPlugDoorPairTarget::Open,
                DoorControlMode::Close => ElectricSlidingPlugDoorPairTarget::Close,
            }
        } else {
            ElectricSlidingPlugDoorPairTarget::NoEnergy
        });

        wait::next_tick().await;
    }
}

#[derive(Builder, Clone)]
pub struct DoorControlProperties {
    #[builder(into)]
    pub request_time: f32,
    #[builder(into)]
    pub warning_time: f32,
    #[builder(into)]
    pub set_system_active: Shared<bool>,
    #[builder(into)]
    pub set_request: Shared<bool>,
    #[builder(into)]
    pub set_released: Shared<bool>,
    #[builder(into)]
    pub set_force: Option<Shared<DoorControlMode>>,
    #[builder(into)]
    pub set_door_closed: Shared<ElectricSlidingPlugDoorPairPositionState>,
    #[builder(into)]
    pub get_door_target: Shared<ElectricSlidingPlugDoorPairTarget>,
}

#[derive(Clone, Default, Debug)]
pub struct DoorControlState {
    pub warning: Shared<bool>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Copy)]
pub enum DoorControlMode {
    #[default]
    Automatic,
    Open,
    Close,
}

pub fn add_door_warning_outside_relay_with_stop_on_speed(
    prop: DoorWarningOutsideRelayWithStopOnSpeedProperties,
) -> DoorWarningOutsideRelayWithStopOnSpeedState {
    let timer_set = Shared::<TimerSet>::default();

    spawn(door_warning_outside_relay_with_stop_on_speed_set_timer(
        prop.released,
        prop.speed,
        timer_set.clone(),
        prop.all_doors_closed,
        prop.max_speed,
    ));

    let timer = add_timer(
        TimerProperties::builder()
            .time(prop.timer_after_closed)
            .set(timer_set)
            .build(),
        None,
    );

    let warning = add_inverter(timer, None);

    DoorWarningOutsideRelayWithStopOnSpeedState { warning }
}

async fn door_warning_outside_relay_with_stop_on_speed_set_timer(
    released: Shared<bool>,
    speed: Shared<f32>,
    timer_set: Shared<TimerSet>,
    doors_closed: Shared<bool>,
    max_speed: f32,
) {
    let mut prev_released = false;
    loop {
        let new_released = released.get();

        if (!doors_closed.get() || prev_released) && !new_released {
            timer_set.set_only_on_change(TimerSet::Hold);
        } else if speed.get() > max_speed || new_released {
            timer_set.set_only_on_change(TimerSet::Reset);
        } else {
            timer_set.set_only_on_change(TimerSet::LetRun);
        }

        if new_released != prev_released {
            wait::ticks(5).await;
        }

        prev_released = new_released;

        wait::next_tick().await;
    }
}

#[derive(Builder, Clone)]
pub struct DoorWarningOutsideRelayWithStopOnSpeedProperties {
    #[builder(into)]
    pub timer_after_closed: f32,
    #[builder(into)]
    pub max_speed: f32,
    #[builder(into)]
    pub released: Shared<bool>,
    #[builder(into)]
    pub all_doors_closed: Shared<bool>,
    #[builder(into)]
    pub speed: Shared<f32>,
}

#[derive(Clone, Default, Debug)]
pub struct DoorWarningOutsideRelayWithStopOnSpeedState {
    pub warning: Shared<bool>,
}
