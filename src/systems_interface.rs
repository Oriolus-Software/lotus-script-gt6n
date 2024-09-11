use lotus_rt::spawn;

use crate::cockpit::ChannelsCockpit;

pub struct InterfaceChannels {
    pub channels_cockpit: ChannelsCockpit,
}

pub fn add_systems_interface(channels: InterfaceChannels) {
    spawn(async move { loop {} });
}
