use lotus_extra::messages::std_messages::MsgLight;
use lotus_rt_extra::{
    shared::Shared, sounds::StartStopSoundProperties, timers::BlinkRelayProperties,
    vehicle_systems::BlinkerState,
};
use lotus_script::prelude::MessageTarget;

const BLINKER_FIRST_ON_TIME: f32 = 0.2;
const BLINKER_FIRST_OFF_TIME: f32 = 0.56;
const BLINKER_ON_TIME: f32 = 0.32;
const BLINKER_OFF_TIME: f32 = 0.43;

#[derive(Default, Debug, Clone)]
pub struct LightState {
    pub voltage: Shared<f32>,
    pub fahrgastraum: Shared<bool>,
    pub stand: Shared<bool>,
    pub abblend: Shared<bool>,
    pub fern: Shared<bool>,
    pub rueck: Shared<bool>,
    pub rueckfahr: Shared<bool>,
    pub brems: Shared<bool>,
    pub blinker_state: Shared<BlinkerState>,
    pub blinker_lampe_rechts: Shared<bool>,
    pub blinker_lampe_links: Shared<bool>,
    pub lm_warnblinker: Shared<bool>,
    pub cockpit_main: Shared<bool>,
    pub cockpit_begleiter: Shared<bool>,
    pub instrumente: Shared<bool>,
}

pub fn add_lights() -> LightState {
    let lights = LightState::default();
    {
        let lights = lights.clone();

        lights
            .cockpit_main
            .relay(&lights.voltage)
            .var_writer("A_CP_FstBelMain");

        lights
            .cockpit_begleiter
            .relay(&lights.voltage)
            .var_writer("A_CP_FstBelBegleiter");

        lights
            .instrumente
            .relay(&lights.voltage)
            .var_writer("A_CP_InstrBel");

        lights
            .fahrgastraum
            .relay(&lights.voltage)
            .var_writer("Fahrgastraumbeleuchtung");

        lights
            .stand
            .relay(&lights.voltage)
            .var_writer("Standlicht")
            .process(|&value| MsgLight { value })
            .send_message(MessageTarget::Broadcast {
                across_couplings: false,
                include_self: false,
            });

        lights
            .abblend
            .relay(&lights.voltage)
            .var_writer("Abblendlicht");

        lights.fern.relay(&lights.voltage).var_writer("Fernlicht");

        lights.rueck.relay(&lights.voltage).var_writer("Ruecklicht");

        lights
            .rueckfahr
            .relay(&lights.voltage)
            .var_writer("Rueckfahrlicht");

        lights.brems.relay(&lights.voltage).var_writer("Bremslicht");

        let blinker_lights_state = lights.blinker_state.blinker(
            BlinkRelayProperties::builder()
                .interval(BLINKER_ON_TIME + BLINKER_OFF_TIME)
                .on_time(BLINKER_ON_TIME)
                .reset_time(BLINKER_ON_TIME - BLINKER_FIRST_ON_TIME)
                .first_off_time_delta(BLINKER_FIRST_OFF_TIME - BLINKER_OFF_TIME)
                .build(),
        );

        blinker_lights_state
            .left
            .forward(&lights.blinker_lampe_links);

        blinker_lights_state
            .right
            .forward(&lights.blinker_lampe_rechts);

        blinker_lights_state.warning.forward(&lights.lm_warnblinker);

        lights
            .blinker_lampe_links
            .to_float()
            .var_writer("BlinkerLeft");

        lights
            .blinker_lampe_rechts
            .to_float()
            .var_writer("BlinkerRight");

        blinker_lights_state.blinker_relay.start_stop_sound(
            StartStopSoundProperties::builder()
                .start_sound("Snd_Relais_Blinker_On".to_string())
                .stop_sound("Snd_Relais_Blinker_Off".to_string())
                .build(),
        );
    }
    lights
}
