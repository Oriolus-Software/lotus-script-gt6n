use lotus_rt_extra::{
    backbone::VehicleBackbone,
    cockpit_simple::{TimedButtonProperties, timed_button},
};

use crate::backbone_special_types;

// #[derive(Debug, Clone)]
// pub struct PassengerElementsState {
//     pub door_buttons: Vec<Shared<bool>>,
// }

pub fn add_passenger_elements(backbone: &mut VehicleBackbone) {
    (0..4).for_each(|i| {
        backbone.insert(
            backbone_special_types::PassengerDoorButtons::DoorRight(i as i8),
            timed_button(
                TimedButtonProperties::builder()
                    .input_event(format!("DoorButton{}", i + 1))
                    .time_staying_on(2.0)
                    .time_before_pressable_again(1.0)
                    .build(),
            ),
        );
    });
}
