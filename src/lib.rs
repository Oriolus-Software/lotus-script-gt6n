use lotus_extra::messages;
use lotus_rt::{spawn, wait};
use lotus_rt_extra::messages::process_message_rt_handler;
use lotus_script::{
    Script,
    content::ContentId,
    graphics::textures::{AlphaMode, TextureAction, TextureCreationOptions},
    log,
    math::UVec2,
    message::Coupling,
    prelude::Texture,
    script,
    time::{self},
    var::{get_var, set_var},
    vehicle::{Axle, RailQuality},
};

use crate::systems_interface::Interface;

pub mod cockpit;
pub mod cockpit_types;
pub mod couplings;
pub mod doors;
pub mod examples;
pub mod input;
pub mod lights;
pub mod misc;
pub mod passenger_elements;
pub mod systems_interface;
pub mod traction;

script!(ScriptGt6n);

pub struct ScriptGt6n {
    test_tex: Option<Texture>,
    interface: Interface,
    written_tex: bool,
}

impl Default for ScriptGt6n {
    fn default() -> Self {
        log::info!("init -----------------------------");

        set_var("veh_number", "2143");

        log::info!("is_coupled: {}", Coupling::is_coupled(&Coupling::Front));
        log::info!("is_coupled: {}", Coupling::is_coupled(&Coupling::Rear));

        Self {
            test_tex: None,
            interface: Interface::default(),
            written_tex: false,
        }
    }
}

impl Script for ScriptGt6n {
    fn init(&mut self) {
        test_message();

        let mut t = Texture::create(TextureCreationOptions {
            width: 256,
            height: 256,
            data: None,
            mipmaps: true,
        });

        //-----------------------------------------

        t.apply_to("TexID_veh_number_black");

        //-----------------------------------------

        self.test_tex = Some(t);

        //-----------------------------------------

        log::info!(
            "cockpit index: {:?}, module_slot_index: {:?}, module_slot_in_class_index: {:?}",
            lotus_script::module::module_slot_cockpit_index(),
            lotus_script::module::module_slot_index(),
            lotus_script::module::module_slot_index_in_class_group(),
        );
    }

    fn tick(&mut self) {
        lotus_rt::tick();

        // set_var("Snd_Traction_A", get_var::<f32>("M_Axle_N_0_1").abs());
        // set_var("Snd_Traction_C", get_var::<f32>("M_Axle_N_1_1").abs());
        // set_var("Snd_Traction_B", get_var::<f32>("M_Axle_N_2_0").abs());

        // 1.0.set("Snd_Fiep_tief");

        set_var("loadforce_Axle_N_1_1", 100000000.0);

        set_var("v_Axle_mps_0_0_abs", get_var::<f32>("v_Axle_mps_0_0").abs());
        set_var("v_Axle_mps_0_1_abs", get_var::<f32>("v_Axle_mps_0_1").abs());
        set_var("v_Axle_mps_2_0_abs", get_var::<f32>("v_Axle_mps_2_0").abs());
        set_var("v_Axle_mps_2_1_abs", get_var::<f32>("v_Axle_mps_2_1").abs());

        // log::info!("B: {}", get_var::<f32>("ZStellung_B"));

        weichensounds();

        if !self.written_tex {
            self.test_tex
                .as_mut()
                .unwrap()
                .add_action(TextureAction::DrawText {
                    font: ContentId {
                        user_id: 3473612,
                        sub_id: 893621505,
                    },
                    text: "Hallo".to_string(),
                    top_left: UVec2 { x: 20, y: 20 },
                    letter_spacing: 0,
                    full_color: Some(lotus_script::graphics::Color {
                        r: 20,
                        g: 20,
                        b: 255,
                        a: 255,
                    }),
                    alpha_mode: AlphaMode::Opaque,
                });
            self.written_tex = true;
        }
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        log::info!("Message: {:?}", msg);
        // process_message_rt_handler(msg);
    }
}

fn test_message() {
    // if !Coupling::is_coupled(&Coupling::Front) {
    //     lotus_script::prelude::send_message(
    //         &messages::Light(20.0),
    //         lotus_script::message::MessageTarget::Broadcast {
    //             across_couplings: false,
    //             include_self: false,
    //         },
    //     );
    // }

    lotus_script::prelude::send_message(
        &messages::Batteryvoltage::On(1.0),
        lotus_script::message::MessageTarget::Broadcast {
            across_couplings: true,
            include_self: true,
        },
    );

    lotus_script::prelude::send_message(
        &messages::PowerSignal::On {
            quickstart: false,
            cabin_id: messages::PowerSignalCabin::ACab,
        },
        lotus_script::message::MessageTarget::Broadcast {
            across_couplings: true,
            include_self: true,
        },
    );
}

fn weichensounds() {
    let (Ok(axle_0), Ok(axle_1)) = (Axle::get(0, 0), Axle::get(0, 1)) else {
        log::error!("Axle 0, 0 or 0, 1 not found");
        return;
    };

    let (quality_a, quality_b) = (axle_0.rail_quality(), axle_1.rail_quality());

    if quality_a == RailQuality::FroggySmooth
        || quality_b == RailQuality::FroggySmooth
        || quality_a == RailQuality::FroggyRough
        || quality_b == RailQuality::FroggyRough
        || quality_a == RailQuality::FlatGroove
        || quality_b == RailQuality::FlatGroove
    {
        set_var("Snd_Rumpeln_Weiche1", 1.0);
    } else {
        set_var("Snd_Rumpeln_Weiche1", 0.0);
    }

    let v = get_var::<f32>("v_Axle_mps_0_0");
    set_var("Snd_Rumpeln_Pitch", 0.9 + v.abs() / 18.0);
}
