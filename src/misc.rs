use lotus_rt_extra::{shared::Shared, simple::StartLoopStopSoundProperties};

#[derive(Default, Debug, Clone)]
pub struct MiscState {
    pub klingel: Shared<bool>,
}

pub fn add_misc() -> MiscState {
    let channels = MiscState::default();
    let c = channels.clone();

    c.klingel.start_loop_stop_sound(
        StartLoopStopSoundProperties::builder()
            .loop_sound("Snd_Klingel_Loop".to_string())
            .stop_sound("Snd_Klingel_End".to_string())
            .build(),
    );

    channels
}
