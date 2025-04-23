use lotus_rt::{spawn, wait};

use crate::standard_elements::Shared;

use super::simple::{
    add_blink_relais, add_bool_to_float_var_unit, add_bool_to_sound_unit, BlinkRelaisProperties,
    BlinkRelaisState, BoolToFloatVarUnitProperties, BoolToSoundUnitProperties,
};

pub fn add_blink_relais_with_light_and_sound(
    prop: BlinkRelaisWithLightAndSoundProperties,
) -> BlinkRelaisState {
    let state = BlinkRelaisState::default();

    let blink_relais = add_blink_relais(prop.blink_relais_properties, None);

    add_bool_to_float_var_unit(
        BoolToFloatVarUnitProperties::builder()
            .float(prop.light_and_sound.light)
            .set_bool(blink_relais.on.clone())
            .build(),
    );

    add_bool_to_sound_unit(
        BoolToSoundUnitProperties::builder()
            .sound(prop.light_and_sound.sound)
            .set_bool(blink_relais.on)
            .build(),
    );

    state
}

#[derive(Clone)]
pub struct LightAndSoundVarPair {
    pub light: String,
    pub sound: String,
}

#[derive(Clone)]
pub struct BlinkRelaisWithLightAndSoundProperties {
    pub blink_relais_properties: BlinkRelaisProperties,
    pub light_and_sound: LightAndSoundVarPair,
}

pub fn add_blink_relais_multiple_entries(prop: BlinkRelaisMultipleEntriesProperties) {
    let running: Shared<bool> = Shared::new(false);
    {
        let running = running.clone();
        let prop = prop.clone();

        spawn(async move {
            loop {
                running.set_only_on_change(prop.entries.iter().any(|(set_bool, _)| set_bool.get()));
                wait::next_tick().await;
            }
        });
    }

    let blinker = add_blink_relais(
        BlinkRelaisProperties::builder()
            .interval(prop.interval)
            .on_time(prop.on_time)
            .set_running(running)
            .build(),
        None,
    );

    for (set_bool, light_and_sound) in prop.entries.iter() {
        let on = blinker.on.clone();

        {
            let blinker_on = blinker.on.clone();
            let set_bool = set_bool.clone();

            let on = on.clone();

            spawn(async move {
                let mut prev_on = false;
                loop {
                    let new_on = set_bool.get() && blinker_on.get();
                    if new_on != prev_on {
                        on.set(new_on);
                        prev_on = new_on;
                    }

                    wait::next_tick().await;
                }
            });
        }

        add_bool_to_float_var_unit(
            BoolToFloatVarUnitProperties::builder()
                .float(light_and_sound.light.clone())
                .set_bool(on.clone())
                .build(),
        );
        add_bool_to_sound_unit(
            BoolToSoundUnitProperties::builder()
                .sound(light_and_sound.sound.clone())
                .set_bool(on.clone())
                .build(),
        );
    }
}

#[derive(Clone)]
pub struct BlinkRelaisMultipleEntriesProperties {
    pub interval: f32,
    pub on_time: f32,
    pub reset_time: Option<f32>,
    pub entries: Vec<(Shared<bool>, LightAndSoundVarPair)>,
}
