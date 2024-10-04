use lotus_rt::{spawn, wait};
use lotus_script::var::VariableType;

use crate::standard_elements::Shared;

#[derive(Default, Debug, Clone)]
pub struct ChannelsLights {
    pub voltage: Shared<f32>,
    pub fahrgastraum: Shared<bool>,
    pub stand: Shared<bool>,
    pub abblend: Shared<bool>,
    pub fern: Shared<bool>,
    pub rueck: Shared<bool>,
    pub rueckfahr: Shared<bool>,
    pub brems: Shared<bool>,
    pub cockpit_main: Shared<bool>,
    pub cockpit_begleiter: Shared<bool>,
    pub instrumente: Shared<bool>,
}

pub fn add_lights() -> ChannelsLights {
    let channels = ChannelsLights::default();
    let c = channels.clone();

    spawn(async move {
        loop {
            fn set_light(c: &ChannelsLights, b: &Shared<bool>, variable: &str) {
                c.voltage.switch(b.get()).set(variable);
            }

            set_light(&c, &c.fahrgastraum, "Fahrgastraumbeleuchtung");
            set_light(&c, &c.stand, "Standlicht");
            set_light(&c, &c.abblend, "Abblendlicht");
            set_light(&c, &c.fern, "Fernlicht");
            set_light(&c, &c.rueck, "Ruecklicht");
            set_light(&c, &c.rueckfahr, "Rueckfahrlicht");
            set_light(&c, &c.brems, "Bremslicht");
            set_light(&c, &c.cockpit_main, "A_CP_FstBelMain");
            set_light(&c, &c.cockpit_begleiter, "A_CP_FstBelBegleiter");
            set_light(&c, &c.instrumente, "A_CP_InstrBel");

            wait::next_tick().await;
        }
    });

    channels
}
