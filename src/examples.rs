use lotus_rt_extra::{
    cockpit_simple::{ButtonProperties, std_button},
    input::InputEvent,
};

fn example_1() {
    let properties = ButtonProperties {
        input_event: Some(InputEvent::new("Key_Reverser_L", 1)),
        animation_var: Some("Schluessel_H_turned".to_string()),
        sound_on: Some("Snd_CP_B_BtnDn".to_string()),
        sound_off: Some("Snd_CP_B_BtnUp".to_string()),
        locked: None,
    };

    let button = std_button(properties);
}
