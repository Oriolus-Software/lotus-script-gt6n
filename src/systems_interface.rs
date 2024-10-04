use lotus_rt::{spawn, wait};

use crate::{
    cockpit::{ChannelsCockpit, RichtungswenderState},
    traction::{ChannelsTraction, TractionDirection},
};

#[derive(Debug, Clone)]
pub struct InterfaceChannels {
    pub channels_cockpit: ChannelsCockpit,
    pub channels_traction: ChannelsTraction,
}

pub fn add_systems_interface(channels: InterfaceChannels) {
    spawn(async move {
        loop {
            let r = channels.channels_cockpit.richtungswender.get();
            let s = channels.channels_cockpit.sollwertgeber.get();

            let (cockpit_a_active, cockpit_a_drive) = match r {
                RichtungswenderState::O => (false, false),
                RichtungswenderState::I => (true, false),
                _ => (true, true),
            };

            channels
                .channels_traction
                .direction
                .set(if cockpit_a_active {
                    match r {
                        RichtungswenderState::V => TractionDirection::Forward,
                        RichtungswenderState::R => TractionDirection::Backward,
                        _ => TractionDirection::Neutral,
                    }
                } else {
                    TractionDirection::Neutral
                });

            channels.channels_traction.target.set(if cockpit_a_active {
                if s < 0.0 {
                    s * 1.111
                } else {
                    s
                }
            } else {
                0.0
            });

            let federspeicher_target = !cockpit_a_drive
                || (cockpit_a_active && channels.channels_cockpit.federspeicher_overwrite.get());

            spawn(set_federspeicher(
                channels.channels_traction.clone(),
                federspeicher_target,
            ));

            wait::next_tick().await;

            channels
                .channels_cockpit
                .lm_federspeicher
                .set(channels.channels_traction.federspeicher.get() && cockpit_a_active);
        }
    });
}

pub async fn set_federspeicher(channels_traction: ChannelsTraction, new_value: bool) {
    wait::seconds(0.3).await;
    channels_traction.federspeicher.set(new_value);
}
