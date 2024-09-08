use cockpit::{add_richtungswender, add_sollwertgeber, RichtungswenderState};
use lotus_rt::sync::watch;
use lotus_script::{script, var::VariableType, Script};
use tech_elements::{
    add_button, add_button_twosided_springloaded, ButtonProperties, ButtonTwoSidedSpringLoaded,
};
use traction::add_traction;

pub mod cockpit;
pub mod input;
pub mod tech_elements;
pub mod traction;

script!(ScriptGt6n);

#[derive(Default)]
pub struct ScriptGt6n {}

struct ReceiverFromCockpit {
    richtungswender: watch::Receiver<RichtungswenderState>,
    sollwertgeber: watch::Receiver<f32>,
}

fn add_cockpit() -> ReceiverFromCockpit {
    let (rw_lock_t, rw_lock_r) = lotus_rt::sync::watch::channel(false);

    let richtungswender = add_richtungswender(rw_lock_r);
    let sollwertgeber = add_sollwertgeber(richtungswender.clone(), rw_lock_t);

    add_button_twosided_springloaded(ButtonTwoSidedSpringLoaded {
        input_event_plus: "HighVoltageMainSwitchOn".into(),
        input_event_minus: "HighVoltageMainSwitchOff".into(),
        animation_var: Some("A_CP_SW_Hauptschalter".into()),
        sound_on: Some("Snd_CP_A_RotBtnOn".into()),
        sound_off: Some("Snd_CP_A_RotBtnOff".into()),
    });

    add_button(ButtonProperties {
        input_event: "Lightcheck".into(),
        animation_var: Some("A_CP_TS_Lampentest".into()),
        sound_on: Some("Snd_CP_A_BtnDn".into()),
        sound_off: Some("Snd_CP_A_BtnUp".into()),
    });

    ReceiverFromCockpit {
        richtungswender,
        sollwertgeber,
    }
}

impl Script for ScriptGt6n {
    fn init(&mut self) {
        let cockpit_receiver = add_cockpit();

        add_traction(
            cockpit_receiver.sollwertgeber,
            cockpit_receiver.richtungswender,
        );

        // let vardiewirunbedingtbrauchen = ContentId {
        //     user_id: 1000,
        //     sub_id: 300008,
        //     version: 0.0,
        // };

        // vardiewirunbedingtbrauchen.set("TexID_veh_number_white");

        // let mut t = Texture::create(TextureCreationOptions {
        //     width: 256,
        //     height: 256,
        //     data: None,
        // });

        // t.apply_to("TexID_veh_number_black");

        // t.add_action(textures::TextureAction::DrawRect(
        //     UVec2 { x: 20, y: 20 },
        //     UVec2 { x: 200, y: 200 },
        //     lotus_script::graphics::Color {
        //         r: 200,
        //         g: 255,
        //         b: 0,
        //         a: 255,
        //     },
        // ))
    }

    // fn actions() -> Vec<RegisterAction> {
    //     Vec::new()
    // }

    fn tick(&mut self) {
        // process_inputs();

        lotus_rt::tick();

        (f32::get("v_Axle_mps_0_1").abs()).set("v_Axle_mps_0_1_abs");

        // self.traction
        //     .apply(self.cockpit.target_traction(), self.cockpit.target_brake());

        // self.timer += delta();

        // let speed = f32::get("v_Axle_mps_0_0").abs();

        // 0.0.set("Snd_Rumpeln_Weiche1");
        // 1.0.set("Snd_Rumpeln_Pitch");
        // 100000000.0.set("Snd_Traction_A");
        // 100000000.0.set("Snd_Traction_B");
        // 100000000.0.set("Snd_Traction_C");
        // 1.0.set("Snd_BrakeFlirr");

        // speed.set("v_Axle_mps_0_0_abs");
        // speed.set("v_Axle_mps_0_1_abs");
        // speed.set("v_Axle_mps_2_0_abs");
        // speed.set("v_Axle_mps_2_1_abs");
    }

    // fn on_message(&mut self, msg: lotus_script::message::Message) {
    //     msg.handle(test_message_handle).ok();
    // }
}
