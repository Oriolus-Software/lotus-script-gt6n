use lotus_extra::vehicle::CockpitSide;
use lotus_rt_extra::{
    backbone::VehicleBackbone, backbone_types, sounds::StartLoopStopSoundProperties,
};

pub fn add_misc(backbone: &mut VehicleBackbone) {
    backbone
        .create_observer(backbone_types::MiscBools::Bell(CockpitSide::A))
        .start_loop_stop_sound(
            StartLoopStopSoundProperties::builder()
                .loop_sound("Snd_Klingel_Loop".to_string())
                .stop_sound("Snd_Klingel_End".to_string())
                .build(),
        );
}
