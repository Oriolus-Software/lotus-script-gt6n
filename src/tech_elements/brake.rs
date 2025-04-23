use bon::Builder;
use lotus_rt::{spawn, wait};
use lotus_script::var::VariableType;

use crate::standard_elements::{exponential_approach, Shared};

use super::simple::{
    add_delay_relay, add_start_loop_stop_sound, DelayRelayProperties, StartLoopStopSoundProperties,
};

pub fn add_brake_combination(prop: BrakeCombinationProperties) {
    spawn(async move {
        let mut values = vec![0.0; prop.elements.len()];

        loop {
            for (element, value) in prop.elements.iter().zip(values.iter_mut()) {
                *value = exponential_approach(
                    *value,
                    element.exponent,
                    element.set_brake.get() * element.reference_force,
                );
            }

            values.iter().sum::<f32>().set(&prop.variable);

            wait::next_tick().await;
        }
    });
}

#[derive(Builder)]
pub struct BrakeCombinationProperties {
    #[builder(into)]
    pub variable: String,
    #[builder(into)]
    pub elements: Vec<BrakeCombinationElement>,
}

#[derive(Builder)]
pub struct BrakeCombinationElement {
    #[builder(into)]
    pub reference_force: f32,
    #[builder(into)]
    pub exponent: f32,
    #[builder(into)]
    pub set_brake: Shared<f32>,
}

pub fn add_rail_brake(prop: RailBrakeProperties) {
    let mut active = false;

    let force_variable = format!("F_RailBrake_Bogie_N_{}", prop.bogie_index);

    spawn(async move {
        loop {
            let voltage = prop.set_voltage.get();

            let new_active = prop.set_active.get() && voltage >= prop.min_voltage.unwrap_or(0.0);

            let force = if new_active {
                prop.reference_force * voltage
            } else {
                0.0
            };

            force.set(&force_variable);

            if new_active != active {
                if let Some(ref variable_sound_volume) = prop.variable_sound_volume {
                    if new_active { 1.0 } else { -0.5 }.set(variable_sound_volume);
                }

                if new_active {
                    if let Some(ref variable_sound_control) = prop.variable_sound_control {
                        true.set(variable_sound_control);
                    }
                };

                active = new_active;
            }

            if active {
                if let Some(ref variable_sound_pitch) = prop.variable_sound_pitch {
                    let speed = f32::get("v_Axle_mps_0_0").abs();

                    (prop.sound_pitch_base + prop.sound_pitch_per_mps * speed)
                        .set(variable_sound_pitch);
                }
            }

            wait::next_tick().await;
        }
    });
}

#[derive(Builder)]
pub struct RailBrakeProperties {
    #[builder(into)]
    pub reference_force: f32,
    #[builder(into)]
    pub min_voltage: Option<f32>,
    #[builder(into)]
    pub sound_pitch_base: f32,
    #[builder(into)]
    pub sound_pitch_per_mps: f32,
    #[builder(into)]
    pub bogie_index: usize,
    #[builder(into)]
    pub variable_sound_volume: Option<String>,
    #[builder(into)]
    pub variable_sound_pitch: Option<String>,
    #[builder(into)]
    pub variable_sound_control: Option<String>,
    #[builder(into)]
    pub set_active: Shared<bool>,
    #[builder(into)]
    pub set_voltage: Shared<f32>,
}

pub fn add_sanding_unit(prop: SandingUnitProperties) {
    let system_variable = format!("sanding_{}_{}", prop.bogie_index, prop.axle_index);

    spawn(async move {});

    let effect = add_delay_relay(
        DelayRelayProperties::builder()
            .on_delay(1.0)
            .off_delay(0.0)
            .set(prop.set_active.clone())
            .build(),
        None,
    );

    if let (Some(sound_start), Some(sound_loop), Some(sound_stop)) =
        (prop.sound_start, prop.sound_loop, prop.sound_stop)
    {
        add_start_loop_stop_sound(
            StartLoopStopSoundProperties::builder()
                .start_sound(sound_start)
                .loop_sound(sound_loop)
                .stop_sound(sound_stop)
                .set_active(prop.set_active.clone())
                .build(),
        );
    }

    effect.on_change(
        move |active| {
            active.set(&system_variable);
        },
        "sanding_unit effect".to_string(),
    );
}

#[derive(Builder)]
pub struct SandingUnitProperties {
    #[builder(into)]
    pub bogie_index: usize,
    #[builder(into)]
    pub axle_index: usize,
    #[builder(into)]
    pub sound_start: Option<String>,
    #[builder(into)]
    pub sound_loop: Option<String>,
    #[builder(into)]
    pub sound_stop: Option<String>,
    #[builder(into)]
    pub set_active: Shared<bool>,
}
