use std::sync::Arc;

use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::{time, var::VariableType};

use crate::standard_elements::Shared;

pub fn add_inertion_slider(prop: InertionSliderProperties) -> InertionSliderState {
    let state = InertionSliderState::default();

    {
        let state = state.clone();

        spawn(async move {
            loop {
                let delta;
                let time_delta = time::delta();

                if state.grabbing.get() {
                    delta = state.grabbing_delta.get();
                    state.velocity.set_only_on_change(delta / time_delta);
                } else {
                    let current_velocity = state.velocity.get();
                    let additional_force = state.additional_force.get();
                    let friction_delta = prop.friction * time_delta;

                    let force_adjusted_velocity = current_velocity + additional_force * time_delta;

                    let new_velocity = if force_adjusted_velocity.abs() <= friction_delta {
                        0.0
                    } else {
                        force_adjusted_velocity - friction_delta * force_adjusted_velocity.signum()
                    };

                    state.velocity.set_only_on_change(new_velocity);
                    delta = new_velocity * time_delta;
                }

                state
                    .position
                    .set_only_on_change(state.position.get() + delta);

                for (index, bump) in prop.bumps.iter().enumerate() {
                    if let Some(bump_position) = bump.position {
                        let sign = if index == 0 { -1.0 } else { 1.0 };

                        // The multiplication with sign is necessary to reverse the inequality sign if needed
                        if sign * state.position.get() > sign * bump_position {
                            state.position.set(bump_position);

                            let new_velocity;
                            if let Some(factor) = bump.factor {
                                new_velocity = state.velocity.get() * factor.max(0.0) * -sign;
                            } else {
                                new_velocity = 0.0;
                            };

                            state.velocity.set(new_velocity);

                            if let Some(ref sound) = bump.sound {
                                true.set(sound.as_str());
                            }

                            if let (Some(ref sound_vol), Some(vol_factor)) =
                                (&bump.sound_vol, bump.sound_vol_factor)
                            {
                                let impact_velocity = state.velocity.get().abs();
                                let volume = impact_velocity * vol_factor;
                                volume.set(sound_vol.as_str());
                            }

                            if let Some(ref on_bump) = prop.on_bump {
                                if let Some(bump_type) = InertionSliderBump::from_index(index) {
                                    on_bump(bump_type);
                                }
                            }
                        }
                    }
                }

                wait::next_tick().await;
            }
        });
    }

    state
}

pub enum InertionSliderBump {
    Lower,
    Upper,
}

#[derive(Builder, Clone)]
pub struct InertionSliderProperties {
    #[builder(into)]
    pub friction: f32,
    #[builder(into)]
    pub on_bump: Option<Arc<dyn Fn(InertionSliderBump) + Send + Sync + 'static>>,
    #[builder(into)]
    pub bumps: [InertionSliderBumpProperties; 2],
}

impl InertionSliderBump {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Lower),
            1 => Some(Self::Upper),
            _ => None,
        }
    }
}

#[derive(Builder, Clone)]
pub struct InertionSliderBumpProperties {
    #[builder(into)]
    pub position: Option<f32>,
    #[builder(into)]
    pub factor: Option<f32>,
    #[builder(into)]
    pub sound: Option<String>,
    #[builder(into)]
    pub sound_vol: Option<String>,
    #[builder(into)]
    pub sound_vol_factor: Option<f32>,
}

#[derive(Clone, Default)]
pub struct InertionSliderState {
    pub position: Shared<f32>,
    pub velocity: Shared<f32>,
    pub grabbing: Shared<bool>,
    pub grabbing_delta: Shared<f32>,
    pub additional_force: Shared<f32>,
}
