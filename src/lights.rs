use lotus_extra::{messages, vehicle::CockpitSide};
use lotus_rt_extra::{
    backbone::VehicleBackbone, backbone_types, sounds::StartStopSoundProperties,
    timers::BlinkRelayProperties,
};
use lotus_script::prelude::MessageTarget;

use crate::backbone_special_types;

const BLINKER_FIRST_ON_TIME: f32 = 0.2;
const BLINKER_FIRST_OFF_TIME: f32 = 0.56;
const BLINKER_ON_TIME: f32 = 0.32;
const BLINKER_OFF_TIME: f32 = 0.43;

pub fn add_lights(backbone: &mut VehicleBackbone) {
    let Some(mut voltage) = backbone.get(backbone_types::ControlVoltage) else {
        return;
    };

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Cockpit(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("A_CP_FstBelMain");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_special_types::Lights::CockpitBegleiter),
            0.0,
            false,
        )
        .var_writer("A_CP_FstBelBegleiter");
    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Instruments(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("A_CP_InstrBel");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Cabin),
            0.0,
            false,
        )
        .var_writer("Fahrgastraumbeleuchtung");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Parking(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("Standlicht")
        .map(|v| messages::std::Light { value: *v })
        .send_message(MessageTarget::Broadcast {
            across_couplings: false,
            include_self: true,
        });

    // l.process::<messages::std::Light>(From::from)
    //     .send_message(MessageTarget::Broadcast {
    //         across_couplings: false,
    //         include_self: true,
    //     });

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::LowBeam(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("Abblendlicht");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::HighBeam(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("Fernlicht");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Tail(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("Ruecklicht");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Reverse(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("Rueckfahrlicht");

    voltage
        .switch(
            &mut backbone.create_observer(backbone_types::LightsTram::Brake(CockpitSide::A)),
            0.0,
            false,
        )
        .var_writer("Bremslicht");

    let mut blinker_lights_state = backbone
        .create_observer(backbone_types::LightBlinkerState)
        .blinker(
            BlinkRelayProperties::builder()
                .interval(BLINKER_ON_TIME + BLINKER_OFF_TIME)
                .on_time(BLINKER_ON_TIME)
                .reset_time(BLINKER_ON_TIME - BLINKER_FIRST_ON_TIME)
                .first_off_time_delta(BLINKER_FIRST_OFF_TIME - BLINKER_OFF_TIME)
                .build(),
        );

    blinker_lights_state
        .left
        .write_to(&backbone.create_observer(backbone_types::LightsTram::BlinkerLeft))
        .to_float()
        .var_writer("BlinkerLeft");

    blinker_lights_state
        .right
        .write_to(&backbone.create_observer(backbone_types::LightsTram::BlinkerRight))
        .to_float()
        .var_writer("BlinkerRight");

    blinker_lights_state.warning.write_to(
        &backbone.create_observer(backbone_types::LightsTram::LmWarningLight(CockpitSide::A)),
    );

    blinker_lights_state.blinker_relay.start_stop_sound(
        StartStopSoundProperties::builder()
            .start_sound("Snd_Relais_Blinker_On".to_string())
            .stop_sound("Snd_Relais_Blinker_Off".to_string())
            .build(),
    );
}
