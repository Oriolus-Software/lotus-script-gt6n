use lotus_rt_extra::{
    simple::{add_start_loop_stop_sound, StartLoopStopSoundProperties},
    standard_elements::Shared,
};

#[derive(Default, Debug, Clone)]
pub struct MiscState {
    pub klingel: Shared<bool>,
}

pub fn add_misc() -> MiscState {
    let channels = MiscState::default();
    let c = channels.clone();

    add_start_loop_stop_sound(
        StartLoopStopSoundProperties::builder()
            .loop_sound("Snd_Klingel_Loop".to_string())
            .stop_sound("Snd_Klingel_End".to_string())
            .set_active(c.klingel)
            .build(),
    );

    channels
}
