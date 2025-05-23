use lotus_rt::{spawn, wait};
use lotus_script::{time::delta, var::set_var};

use lotus_rt_extra::shared::Shared;

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
    let channels = LightState::default();
    let c = channels.clone();

    let mut blinkgeber = BlinkgeberState::default();

    spawn(async move {
        loop {
            fn set_light(c: &LightState, b: &Shared<bool>, variable: &str) {
                set_var(variable, c.voltage.switch(b.get()));
            }

            fn running_blinker(c: &LightState, s: &mut BlinkgeberState) {
                let prev_running = s.running;

                s.running = c.blinker_state.get().is_active();

                if s.running != prev_running && s.running {
                    s.first = true;
                    s.on = true;
                    s.timer = 0.0;
                }

                if s.update(delta()) {
                    if s.on {
                        set_var("Snd_Relais_Blinker_On", true);
                    } else {
                        set_var("Snd_Relais_Blinker_Off", true);
                    }
                }
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

            running_blinker(&c, &mut blinkgeber);

            c.blinker_lampe_links
                .set_only_on_change(blinkgeber.on && c.blinker_state.get().is_links_active());
            c.blinker_lampe_rechts
                .set_only_on_change(blinkgeber.on && c.blinker_state.get().is_rechts_active());
            c.lm_warnblinker
                .set_only_on_change(blinkgeber.on && c.blinker_state.get().is_warn_active());

            set_light(&c, &c.blinker_lampe_rechts, "BlinkerRight");
            set_light(&c, &c.blinker_lampe_links, "BlinkerLeft");

            wait::next_tick().await;
        }
    });

    channels
}

#[derive(Default, Debug, Clone)]
struct BlinkgeberState {
    running: bool,
    on: bool,
    first: bool,
    timer: f32,
}

impl BlinkgeberState {
    fn get_timer_limit(&self) -> f32 {
        match (self.first, self.on) {
            (true, true) => BLINKER_FIRST_ON_TIME,
            (true, false) => BLINKER_FIRST_OFF_TIME,
            (false, true) => BLINKER_ON_TIME,
            (false, false) => BLINKER_OFF_TIME,
        }
    }

    fn update(&mut self, delta: f32) -> bool {
        if !self.running {
            self.on = false;
            return false;
        }

        self.timer += delta;
        if self.timer > self.get_timer_limit() {
            self.on = !self.on;
            self.timer = 0.0;
            if self.on {
                self.first = false;
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlinkerState {
    #[default]
    Aus,
    Links,
    Rechts,
    Warn,
}

impl BlinkerState {
    pub fn is_active(&self) -> bool {
        *self != BlinkerState::Aus
    }

    pub fn is_links_active(&self) -> bool {
        *self == BlinkerState::Links || *self == BlinkerState::Warn
    }

    pub fn is_rechts_active(&self) -> bool {
        *self == BlinkerState::Rechts || *self == BlinkerState::Warn
    }

    pub fn is_warn_active(&self) -> bool {
        *self == BlinkerState::Warn
    }
}
