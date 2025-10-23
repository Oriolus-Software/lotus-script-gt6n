use lotus_rt_extra::{backbone::VehicleBackbone, sounds::StartLoopStopSoundProperties};

use crate::backbone_types;

pub fn add_misc(backbone: &mut VehicleBackbone) {
    backbone
        .create_observer(backbone_types::MiscBools::Klingel)
        .start_loop_stop_sound(
            StartLoopStopSoundProperties::builder()
                .loop_sound("Snd_Klingel_Loop".to_string())
                .stop_sound("Snd_Klingel_End".to_string())
                .build(),
        );
}
