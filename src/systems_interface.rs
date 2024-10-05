use lotus_rt::{spawn, wait};

use crate::{
    cockpit::{ChannelsCockpit, RichtungswenderState},
    lights::ChannelsLights,
    traction::{ChannelsTraction, TractionDirection},
};

#[derive(Debug, Clone)]
pub struct InterfaceChannels {
    pub channels_cockpit: ChannelsCockpit,
    pub channels_traction: ChannelsTraction,
    pub channels_lights: ChannelsLights,
}

pub fn add_systems_interface(channels: InterfaceChannels) {
    // let c = channels.channels_traction.clone();

    // channels.channels_cockpit.lightcheck.on_change(move |&p| {
    //     let c = c.clone();
    //     spawn(set_federspeicher(c, p));
    // });

    spawn(async move {
        channels.channels_lights.voltage.set(1.0);

        loop {
            tick_traction(&channels);
            tick_lights(&channels);

            wait::next_tick().await;
        }
    });
}

pub async fn set_federspeicher(channels_traction: ChannelsTraction, new_value: bool) {
    wait::seconds(0.3).await;
    channels_traction.federspeicher.set(new_value);
}

fn tick_traction(c: &InterfaceChannels) {
    let r = c.channels_cockpit.richtungswender.get();
    let s = c.channels_cockpit.sollwertgeber.get();

    let (cockpit_a_active, cockpit_a_drive) = match r {
        RichtungswenderState::O => (false, false),
        RichtungswenderState::I => (true, false),
        _ => (true, true),
    };

    c.channels_traction.direction.set(if cockpit_a_active {
        match r {
            RichtungswenderState::V => TractionDirection::Forward,
            RichtungswenderState::R => TractionDirection::Backward,
            _ => TractionDirection::Neutral,
        }
    } else {
        TractionDirection::Neutral
    });

    c.channels_traction.target.set(if cockpit_a_active {
        if s < 0.0 {
            s * 1.111
        } else {
            s
        }
    } else {
        0.0
    });

    let federspeicher_target =
        !cockpit_a_drive || (cockpit_a_active && c.channels_cockpit.federspeicher_overwrite.get());

    spawn(set_federspeicher(
        c.channels_traction.clone(),
        federspeicher_target,
    ));

    c.channels_cockpit
        .lm_federspeicher
        .set(c.channels_traction.federspeicher.get() && cockpit_a_active);
}

fn tick_lights(c: &InterfaceChannels) {
    let standlicht = c.channels_cockpit.beleuchtung_aussen.get() > 0;
    c.channels_lights.stand.set(standlicht);
    c.channels_lights.rueck.set(standlicht);

    c.channels_lights
        .abblend
        .set(c.channels_cockpit.beleuchtung_aussen.get() > 1);

    let fernlicht = c.channels_cockpit.beleuchtung_aussen.get() > 2;
    c.channels_lights.fern.set(fernlicht);
    c.channels_cockpit.lm_fernlicht.set(fernlicht);

    c.channels_lights
        .rueckfahr
        .set(c.channels_cockpit.richtungswender.get() == RichtungswenderState::R);

    c.channels_lights
        .brems
        .set(c.channels_cockpit.sollwertgeber.get() < 0.0);

    c.channels_lights
        .cockpit_main
        .set(c.channels_cockpit.beleuchtung_fahrerraum.get() >= 2);

    c.channels_lights
        .cockpit_begleiter
        .set(c.channels_cockpit.beleuchtung_fahrerraum.get() >= 1);
}
