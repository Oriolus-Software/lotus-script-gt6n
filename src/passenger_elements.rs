use lotus_rt_extra::{
    cockpit_simple::{add_timed_button, TimedButtonProperties},
    standard_elements::Shared,
};

#[derive(Debug, Clone)]
pub struct PassengerElementsState {
    pub door_buttons: Vec<Shared<bool>>,
}

pub fn add_passenger_elements() -> PassengerElementsState {
    let door_buttons: Vec<_> = (0..4)
        .map(|i| {
            add_timed_button(
                TimedButtonProperties::builder()
                    .input_event(format!("DoorButton{}", i + 1))
                    .time_staying_on(2.0)
                    .time_before_pressable_again(1.0)
                    .build(),
            )
        })
        .collect();

    PassengerElementsState { door_buttons }
}
