use lotus_rt_extra::{
    backbone::VehicleBackbone,
    brake::{
        BrakeCombinationElement, BrakeCombinationProperties, RailBrakeProperties,
        SandingUnitProperties, brake_combination,
    },
    observer::{Observer, changed},
    traction::{
        ThreePhaseTractionUnitProperties, ThreePhaseTractionUnitState, TractionUnitMode,
        three_phase_traction_unit,
    },
    variables::var_reader,
};
use lotus_script::{log, vehicle::Axle};

use crate::backbone_types;

const VMAX: f32 = 60.0 / 3.6;
const VMAX_BACK: f32 = 15.0 / 3.6;
const V_EBRAKE_LIMIT: f32 = 5.0 / 3.6;
const MAXBRAKEFORCE_N: f32 = 16_000.0;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum TractionDirection {
    Forward,
    #[default]
    Neutral,
    Backward,
}

#[derive(Clone)]
pub struct TractionUnit {
    pub traction_unit: ThreePhaseTractionUnitState,
    pub wheelspeed: Observer<f32>,
    pub mg_relay: Observer<bool>,
}

pub fn add_traction(backbone: &mut VehicleBackbone) {
    let mut direction = backbone.create_observer(backbone_types::TractionDirection);
    let mut target = backbone.create_observer(backbone_types::TractionFloat::Target);
    let mut federspeicher = backbone.create_observer(backbone_types::TractionBool::Federspeicher);
    let mut mg: Observer<bool> = backbone.create_observer(backbone_types::TractionBool::MgBremse);
    let mut speed = backbone.create_observer(backbone_types::TractionFloat::Speed);
    let mut sanden = backbone.create_observer(backbone_types::TractionBool::Sanden);

    let traction_mode = Observer::<TractionUnitMode>::default();

    let target_force = Observer::<f32>::default();

    let traction_unit = |axle: Axle, vehicle_part: String| -> TractionUnit {
        let wheelspeed = var_reader::<f32>(axle.velocity_var_name());

        let mut mg_relay = Observer::<bool>::default();

        let source_voltage = Observer::<f32>::default();

        let mut traction_unit = three_phase_traction_unit(
            ThreePhaseTractionUnitProperties::builder()
                .max_force_acceleration(16_000.0)
                .max_power_acceleration(100_000.0)
                .max_force_braking(MAXBRAKEFORCE_N)
                .max_force_braking_per_speed(10_000.0)
                .delay_exponent(10.0)
                .voltage_min(0.8)
                .set_traction_max_reverse_speed(1.0)
                .set_wheelspeed(wheelspeed.clone())
                .set_target_force(target_force.clone())
                .set_traction_mode(traction_mode.clone())
                .set_source_voltage(source_voltage.clone())
                .build(),
        );

        let mut voltage = Observer::<f32>::default();

        mg_relay
            .delay_relay(0.14, 0.14)
            .to_float()
            .multiply_observer(&mut voltage, 0.0, 1.0)
            .rail_brake(
                RailBrakeProperties::builder()
                    .reference_force(128_000.0)
                    .min_voltage(0.8)
                    .sound_pitch_base(0.8)
                    .sound_pitch_per_mps(0.05)
                    .bogie(axle.bogie())
                    .variable_sound_volume(format!("Snd_Mg_{vehicle_part}_Friction_vol"))
                    .variable_sound_control(format!("Snd_Mg_{vehicle_part}"))
                    .variable_sound_pitch("Snd_Mg_Friction_pitch")
                    .build(),
            );
        traction_unit.wheel_force.for_each(move |force| {
            axle.set_traction_force_newton(*force);
        });

        source_voltage.call(&1.0);

        traction_unit
            .wheel_force
            .var_writer(format!("Snd_Traction_{vehicle_part}"));
        TractionUnit {
            traction_unit,
            wheelspeed,
            mg_relay,
        }
    };

    let create_axle = |bogie_index: usize, axle_index: usize| -> Option<Axle> {
        match Axle::get(bogie_index, axle_index) {
            Ok(axle) => Some(axle),
            Err(e) => {
                log::error!(
                    "Axle not found: bogie_index: {}, axle_index: {}, error: {}",
                    bogie_index,
                    axle_index,
                    e
                );
                None
            }
        }
    };

    let axles_opts = [create_axle(0, 1), create_axle(1, 1), create_axle(2, 0)];

    let Some(axle_1) = axles_opts[0] else {
        return;
    };

    let Some(axle_2) = axles_opts[1] else {
        return;
    };

    let Some(axle_3) = axles_opts[2] else {
        return;
    };

    let axles = [axle_1, axle_2, axle_3];

    let mut traction_units = [
        traction_unit(axles[0], "A".into()),
        traction_unit(axles[1], "C".into()),
        traction_unit(axles[2], "B".into()),
    ];

    sanden.proto_sanding_unit(
        SandingUnitProperties::builder()
            .bogie_index(0_usize)
            .axle_index(1_usize)
            .sound_start("Snd_Sanden_Strt")
            .sound_loop("Snd_Sanden_Loop")
            .sound_stop("Snd_Sanden_Stop")
            .build(),
    );
    sanden.proto_sanding_unit(
        SandingUnitProperties::builder()
            .bogie_index(1_usize)
            .axle_index(1_usize)
            .build(),
    );
    sanden.proto_sanding_unit(
        SandingUnitProperties::builder()
            .bogie_index(2_usize)
            .axle_index(0_usize)
            .build(),
    );

    for traction_unit in traction_units.iter() {
        mg.write_to(&traction_unit.mg_relay);
    }

    let hydraulic_brake_target = Observer::<f32>::default();
    let parking_brake_target = Observer::<f32>::default();

    let add_brake_unit = |axle: Axle| {
        brake_combination(
            BrakeCombinationProperties::builder()
                .axle(axle)
                .elements(vec![
                    BrakeCombinationElement::builder()
                        .reference_force(16_000.0)
                        .exponent(10.0)
                        .set_brake(hydraulic_brake_target.clone())
                        .build(),
                    BrakeCombinationElement::builder()
                        .reference_force(10_000.0)
                        .exponent(10.0)
                        .set_brake(parking_brake_target.clone())
                        .build(),
                ])
                .build(),
        );
    };

    add_brake_unit(axles[0]);
    add_brake_unit(axles[1]);
    add_brake_unit(axles[2]);

    let mut fast_brake = target.less_than_value(-0.95);
    let mut reversed = direction.map(|dir| *dir == TractionDirection::Backward);

    // TODO: add emergency brake
    let mut max_brake = fast_brake.or_value(false);

    let mut ref_speed = var_reader::<f32>("v_Axle_mps_1_0");
    let mut speed_in_dir = direction
        .map(|dir| *dir == TractionDirection::Backward)
        .if_then_o_o(
            &mut ref_speed.additive_inverse(),
            &mut ref_speed,
            false,
            0.0,
            0.0,
        );

    let mut target_is_negative = target.less_than_value(0.0);

    // TODO: Add Schleuderschutz
    let mut abregelung_or_schleuderschutz = reversed
        .not()
        .and_observer(&mut speed_in_dir.greater_than_value(VMAX), true, false)
        .or_observer(
            &mut reversed.and_observer(
                &mut speed_in_dir.greater_than_value(VMAX_BACK),
                false,
                false,
            ),
            false,
            false,
        );

    let mut target_traction = max_brake.if_then_v_o(
        -1.0,
        &mut target_is_negative.if_then_o_o(
            &mut target.multiply_value(1.111),
            &mut abregelung_or_schleuderschutz.if_then_v_o(0.0, &mut target, false, 0.0),
            false,
            0.0,
            0.0,
        ),
        false,
        0.0,
    );

    let mut mode = target_traction.zip(
        &mut reversed,
        |(traction, reversed), next| {
            next(if *traction > 0.01 {
                if *reversed {
                    &TractionUnitMode::Backward
                } else {
                    &TractionUnitMode::Forward
                }
            } else if *traction < -0.01 {
                &TractionUnitMode::Brake
            } else {
                &TractionUnitMode::Off
            })
        },
        0.0,
        false,
    );

    mode.filter(changed()).write_to(&traction_mode);

    let mut mode_acceleration =
        mode.map(|m| *m == TractionUnitMode::Forward || *m == TractionUnitMode::Backward);

    let mut mode_fix_cond_a = mode_acceleration
        .and_observer(&mut target_traction.greater_than_value(0.0), false, false)
        .or_observer(&mut speed_in_dir.greater_than_value(0.1), false, false);

    let mut mode_fix_cond_b =
        speed_in_dir
            .less_than_value(0.1)
            .and_observer(&mut mode_acceleration.not(), false, false);

    let mut mode_fix = mode_fix_cond_a.zip(
        &mut mode_fix_cond_b,
        |(a, b), next| {
            if *a {
                next(&true)
            } else if *b {
                next(&false)
            }
        },
        false,
        false,
    );

    // TODO: Add Gleitschutz
    // if gleitschutz_active && mode == TractionUnitMode::Brake {
    //     target_traction /= 3.0;
    // }

    target_traction.map(|t| t.abs()).write_to(&target_force);

    federspeicher.to_float().write_to(&parking_brake_target);

    mode.map(|m| *m == TractionUnitMode::Brake)
        .if_then_o_v(
            &mut traction_units[0]
                .traction_unit
                .wheel_force
                .map(|f| f.abs() / MAXBRAKEFORCE_N),
            0.0,
            false,
            0.0,
        )
        .var_writer("Snd_BrakeFlirr");

    let mut condition_pneu_max = mode_fix
        .and_observer(&mut federspeicher.not(), true, false)
        .or_observer(&mut max_brake, false, false);

    let mut condition_brake_mode = mode.map(|m| *m == TractionUnitMode::Brake);

    let mut pneu_target = condition_pneu_max.if_then_v_o(
        1.0,
        &mut condition_brake_mode.if_then_o_v(
            &mut target_traction.zip(
                &mut speed,
                |(traction, speed), next| {
                    next(&((*traction).abs() * (1.0 - (*speed).abs() / V_EBRAKE_LIMIT).max(0.0)))
                },
                0.0,
                0.0,
            ),
            0.0,
            false,
            0.0,
        ),
        false,
        0.0,
    );

    // TODO: Add Gleitschutz
    // if gleitschutz_active {
    //     pneu_target /= 3.0;
    // }

    pneu_target.write_to(&hydraulic_brake_target);

    // TODO: Add Additional sounds
}
