use std::f32::NAN;

use lotus_extra::messages::std_messages::{MsgLight, MsgVehNumber};
use lotus_rt_extra::{messages::process_message_rt_handler, shared::Shared};
use lotus_script::{
    Script,
    graphics::textures::{Texture, TextureAction, TextureCreationOptions},
    log,
    math::UVec2,
    message::{Coupling, MessageMeta},
    prelude::MessageType,
    script,
    var::{get_var, set_var},
    vehicle::{Axle, Bogie, RailQuality, acceleration_vs_ground, velocity_vs_ground},
};
use serde::{Deserialize, Serialize};

use crate::systems_interface::Interface;

pub mod cockpit;
pub mod cockpit_types;
pub mod couplings;
pub mod doors;
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
}

impl Default for ScriptGt6n {
    fn default() -> Self {
        Self {
            test_tex: None,
            interface: Interface::default(),
            // test_message_shared: Shared::new(MsgVehNumber {
            //     value: "".to_string(),
            // }),
        }
    }
}

impl Script for ScriptGt6n {
    fn init(&mut self) {
        log::info!("init -----------------------------");

        if Axle::get(0, 0).is_err() {
            log::error!("Axle 0, 0 not found");
        };
        if Axle::get(0, 1).is_err() {
            log::error!("Axle 0, 1 not found");
        };

        if Axle::get(0, 2).is_err() {
            log::error!("Axle 0, 2 not found");
        };
        if acceleration_vs_ground().is_nan() {
            log::error!("Acceleration not found");
        };

        log::info!("is_coupled: {}", Coupling::is_coupled(&Coupling::Front));
        log::info!("is_coupled: {}", Coupling::is_coupled(&Coupling::Rear));

        self.interface = Interface::default();

        //-----------------------------------------

        // let vardiewirunbedingtbrauchen = ContentId {
        //     user_id: 5748540,
        //     sub_id: 110000,
        //     version: 0.0,
        // };

        // vardiewirunbedingtbrauchen.set("TexID_veh_number_white");

        //-----------------------------------------

        // let mut source_t = Texture::create(TextureCreationOptions {
        //     width: 256,
        //     height: 256,
        //     data: None,
        // });

        // source_t.add_action(TextureAction::DrawRect {
        //     start: UVec2 { x: 64, y: 64 },
        //     end: UVec2 { x: 192, y: 192 },
        //     color: lotus_script::graphics::Color {
        //         r: 0,
        //         g: 0,
        //         b: 255,
        //         a: 255,
        //     },
        // });

        // source_t.flush();

        // source_t.apply_to("TexID_veh_number_white");

        let mut t = Texture::create(TextureCreationOptions {
            width: 64,
            height: 64,
            data: None,
        });

        //-----------------------------------------

        t.apply_to("TexID_veh_number_black");

        t.add_action(TextureAction::DrawRect {
            start: UVec2 { x: 1, y: 1 },
            end: UVec2 { x: 63, y: 63 },
            color: lotus_script::graphics::Color {
                r: 255,
                g: 20,
                b: 0,
                a: 255,
            },
        });

        // t.flush();

        // t.add_action(TextureAction::DrawText {
        //     font: ContentId {
        //         user_id: 1000,
        //         sub_id: 210,
        //     },
        //     text: "Hallo".to_string(),
        //     top_left: UVec2 { x: 20, y: 20 },
        //     letter_spacing: 0,

        //     full_color: Some(lotus_script::graphics::Color {
        //         r: 20,
        //         g: 20,
        //         b: 255,
        //         a: 255,
        //     }),
        //     alpha_mode: AlphaMode::Blend,
        // });

        // t.draw_texture(
        //     &source_t,
        //     DrawTextureOpts {
        //         source_rect: Some(lotus_script::math::Rectangle {
        //             start: (UVec2 { x: 62, y: 62 }),
        //             end: (UVec2 { x: 194, y: 194 }),
        //         }),
        //         target_rect: Some(lotus_script::math::Rectangle {
        //             start: (UVec2 { x: 50, y: 50 }),
        //             end: (UVec2 { x: 100, y: 100 }),
        //         }),
        //     },
        // );

        //-----------------------------------------

        self.test_tex = Some(t);

        // self.source_test_tex = Some(source_t);

        //-----------------------------------------
    }

    // fn actions() -> Vec<RegisterAction> {
    //     Vec::new()
    // }

    fn tick(&mut self) {
        // if let Some(f) = lotus_script::font::text_len(
        //     ContentId {
        //         user_id: 1000,
        //         sub_id: 210,
        //         version: 0.0,
        //     },
        //     "Hallo",
        //     0,
        // ) {
        //     log::info!("{:?}", f);
        // }

        // process_inputs();

        lotus_rt::tick();

        // self.traction
        //     .apply(self.cockpit.target_traction(), self.cockpit.target_brake());

        // self.timer += delta();

        // set_var("Snd_Traction_A", get_var::<f32>("M_Axle_N_0_1").abs());
        // set_var("Snd_Traction_C", get_var::<f32>("M_Axle_N_1_1").abs());
        // set_var("Snd_Traction_B", get_var::<f32>("M_Axle_N_2_0").abs());

        // 1.0.set("Snd_Fiep_tief");

        set_var("loadforce_Axle_N_1_1", 100000000.0);

        set_var("v_Axle_mps_0_0_abs", get_var::<f32>("v_Axle_mps_0_0").abs());
        set_var("v_Axle_mps_0_1_abs", get_var::<f32>("v_Axle_mps_0_1").abs());
        set_var("v_Axle_mps_2_0_abs", get_var::<f32>("v_Axle_mps_2_0").abs());
        set_var("v_Axle_mps_2_1_abs", get_var::<f32>("v_Axle_mps_2_1").abs());

        weichensounds();

        // if get_var::<f32>("A_CP_SW_Wischer") > 0.5 {
        //     send_message(
        //         if get_var::<f32>("A_CP_SW_Wischer") > 1.5 {
        //             &BlinkerState::On
        //         } else {
        //             &BlinkerState::Off
        //         },
        //         [MessageTarget::Broadcast {
        //             across_couplings: false,
        //             include_self: true,
        //         }],
        //     )
        // };
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        // msg.handle(|m: BlinkerState| {
        //     match m {
        //         BlinkerState::Off => {
        //             set_var("BlinkerRight", &0.0);
        //         }
        //         BlinkerState::On => {
        //             set_var("BlinkerRight", &1.0);
        //         }
        //     };
        //     Ok(())
        // })
        // .ok();
        // msg.handle(|m| self.test_message_shared.message_handler(m));
        process_message_rt_handler(msg);
    }
}

fn weichensounds() {
    let (quality_a, quality_b) = (
        Axle::get(0, 0).unwrap().rail_quality(),
        Axle::get(0, 1).unwrap().rail_quality(),
    );
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
