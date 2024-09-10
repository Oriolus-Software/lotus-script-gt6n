use crate::cockpit::{ReceiverFromCockpit, SenderToCockpit};

pub struct InterfaceChannels {
    pub cockpit_receiver: ReceiverFromCockpit,
    pub cockpit_sender: SenderToCockpit,
}

pub fn add_systems_interface(channels: InterfaceChannels) {}
