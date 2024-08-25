use cockpit::Cockpit;
use input::process_inputs;
use lotus_script::{
    action::RegisterAction,
    delta,
    graphics::textures::{self, Texture, TextureCreationOptions},
    math::UVec2,
    message::BatterySwitch,
    script,
    var::VariableType,
    Script,
};
use traction::Traction;

pub mod cockpit;
pub mod input;
pub mod tech_elements;
pub mod traction;

script!(ScriptGt6n);

pub struct ScriptGt6n {
    timer: f32,
    cockpit: Cockpit,
    traction: Traction,
}

fn test_message_handle(msg: BatterySwitch) -> Result<(), Box<dyn std::error::Error>> {
    msg.0
        .then_some(1.0)
        .unwrap_or_default()
        .set("A_LM_Fernlicht");
    Ok(())
}

impl Script for ScriptGt6n {
    fn init(&mut self) {
        1.0.set("Snd_Rumpeln_Weiche1");

        // let vardiewirunbedingtbrauchen = ContentId {
        //     user_id: 1000,
        //     sub_id: 300008,
        //     version: 0.0,
        // };

        //vardiewirunbedingtbrauchen.set("TexID_veh_number_black");
        // vardiewirunbedingtbrauchen.set("TexID_veh_number_white");

        let mut t = Texture::create(TextureCreationOptions {
            width: 256,
            height: 256,
            data: None,
        });

        t.apply_to("TexID_veh_number_black");

        t.add_action(textures::TextureAction::DrawRect(
            UVec2 { x: 20, y: 20 },
            UVec2 { x: 200, y: 200 },
            lotus_script::graphics::Color {
                r: 200,
                g: 255,
                b: 0,
                a: 255,
            },
        ))
    }

    fn actions() -> Vec<RegisterAction> {
        Vec::new()
    }

    fn tick(&mut self) {
        process_inputs();

        self.cockpit.tick();
        self.traction
            .apply(self.cockpit.target_traction(), self.cockpit.target_brake());

        self.timer += delta();

        let speed = f32::get("v_Axle_mps_0_0").abs();

        0.0.set("Snd_Rumpeln_Weiche1");
        1.0.set("Snd_Rumpeln_Pitch");
        100000000.0.set("Snd_Traction_A");
        100000000.0.set("Snd_Traction_B");
        100000000.0.set("Snd_Traction_C");
        // 1.0.set("Snd_BrakeFlirr");

        speed.set("v_Axle_mps_0_0_abs");
        speed.set("v_Axle_mps_0_1_abs");
        speed.set("v_Axle_mps_2_0_abs");
        speed.set("v_Axle_mps_2_1_abs");
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        msg.handle(test_message_handle).ok();
    }
}

impl Default for ScriptGt6n {
    fn default() -> Self {
        Self {
            cockpit: Cockpit::new(),
            traction: Traction::default(),
            timer: 0.0,
        }
    }
}
