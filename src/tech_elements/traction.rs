use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::log;

use crate::standard_elements::{exponential_approach, Shared};

pub fn add_three_phase_traction_unit(
    prop: ThreePhaseTractionUnitProperties,
) -> ThreePhaseTractionUnitState {
    let state = ThreePhaseTractionUnitState::default();

    {
        let wheel_force = state.wheel_force.clone();
        let current_force_normalized = state.current_force_normalized.clone();

        spawn(async move {
            let mut force_normalized = 0.0;

            loop {
                let speed = prop.set_wheelspeed.get();
                let direction = prop.set_traction_mode.get();
                let direction_signum: f32 = direction.into();

                let direction = if prop.set_source_voltage.get() < prop.voltage_min
                    || !prop
                        .set_high_voltage_switch
                        .clone()
                        .map_or(true, |v| v.get())
                    || speed * direction_signum < -prop.set_traction_max_reverse_speed
                {
                    TractionUnitMode::Off
                } else {
                    direction
                };

                force_normalized = match direction {
                    TractionUnitMode::Off => 0.0,
                    TractionUnitMode::Forward | TractionUnitMode::Backward => {
                        log::info!("b - prop.delay_exponent: {:?}", prop.delay_exponent);
                        exponential_approach(
                            force_normalized,
                            prop.delay_exponent,
                            prop.set_target_force.get() * direction_signum,
                        )
                    }
                    TractionUnitMode::Brake => exponential_approach(
                        force_normalized,
                        prop.delay_exponent,
                        prop.set_target_force.get() * -1.0 * speed.signum(),
                    ),
                };

                current_force_normalized.set(force_normalized);

                let reference_force = if direction == TractionUnitMode::Brake {
                    (prop.max_force_braking_per_speed * speed.abs()).min(prop.max_force_braking)
                } else {
                    (prop.max_power_acceleration / speed.abs().max(0.001))
                        .min(prop.max_force_acceleration)
                };

                wheel_force.set(force_normalized * reference_force);

                wait::next_tick().await;
            }
        });
    }

    state
}

#[derive(Builder)]
pub struct ThreePhaseTractionUnitProperties {
    /// maximum force (F in N) when accelerating, always positive
    #[builder(into)]
    pub max_force_acceleration: f32,
    /// maximum power (P in W = N*m/s) when accelerating, always positive
    #[builder(into)]
    pub max_power_acceleration: f32,
    /// maximum force (F in N) when braking, always positive
    #[builder(into)]
    pub max_force_braking: f32,
    /// maximum force per speed (F/v in N/(m/s))
    #[builder(into)]
    pub max_force_braking_per_speed: f32,
    /// if the vehicle is rolling against the set direction of the traction unit won't be able to apply force if the (absolute) speed is higher than this value (always positive)
    #[builder(into)]
    pub set_traction_max_reverse_speed: f32,
    #[builder(into)]
    pub delay_exponent: f32,
    #[builder(into)]
    pub voltage_min: f32,
    #[builder(into)]
    pub set_wheelspeed: Shared<f32>,
    /// normalized to reference force, always positive
    #[builder(into)]
    pub set_target_force: Shared<f32>,
    #[builder(into)]
    pub set_traction_mode: Shared<TractionUnitMode>,
    #[builder(into)]
    pub set_source_voltage: Shared<f32>,
    #[builder(into)]
    pub set_high_voltage_switch: Option<Shared<bool>>,
}

#[derive(Debug, Default, Clone)]
pub struct ThreePhaseTractionUnitState {
    pub current_force_normalized: Shared<f32>,
    pub wheel_force: Shared<f32>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TractionUnitMode {
    #[default]
    Off,
    Forward,
    Backward,
    Brake,
}

impl From<TractionUnitMode> for f32 {
    fn from(direction: TractionUnitMode) -> f32 {
        match direction {
            TractionUnitMode::Forward => 1.0,
            TractionUnitMode::Backward => -1.0,
            _ => 0.0,
        }
    }
}
