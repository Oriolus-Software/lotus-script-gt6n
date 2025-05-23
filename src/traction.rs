use lotus_rt::{spawn, wait};

use lotus_rt_extra::{
    brake::{
        BrakeCombinationElement, BrakeCombinationProperties, RailBrakeProperties,
        SandingUnitProperties, brake_combination,
    },
    shared::Shared,
    traction::{
        ThreePhaseTractionUnitProperties, ThreePhaseTractionUnitState, TractionUnitMode,
        three_phase_traction_unit,
    },
};
use lotus_script::var::{get_var, set_var};

const VMAX: f32 = 60.0 / 3.6;
const VMAX_BACK: f32 = 15.0 / 3.6;
const V_EBRAKE_LIMIT: f32 = 5.0 / 3.6;
const MAXBRAKEFORCE_N: f32 = 16_000.0;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TractionDirection {
    Forward,
    Neutral,
    Backward,
}

#[derive(Debug, Clone)]
pub struct TractionState {
    pub direction: Shared<TractionDirection>,
    pub target: Shared<f32>,
    pub federspeicher: Shared<bool>,
    pub speed: Shared<f32>,
    pub mg: Shared<bool>,
    pub sanding: Shared<bool>,
}

#[derive(Debug, Clone)]
pub struct TractionUnit {
    pub traction_unit: ThreePhaseTractionUnitState,
    pub wheelspeed: Shared<f32>,
    pub mg_relay: Shared<bool>,
}

pub fn add_traction() -> TractionState {
    let state = TractionState {
        direction: Shared::new(TractionDirection::Forward),
        target: Shared::new(0.0),
        federspeicher: Shared::new(false),
        mg: Shared::new(false),
        speed: Shared::new(0.0),
        sanding: Shared::new(false),
    };

    let traction_mode = Shared::new(TractionUnitMode::Off);
    let target_force = Shared::new(0.0);

    let traction_unit = |bogie: usize, axle: usize, vehicle_part: String| -> TractionUnit {
        let wheelspeed = Shared::<f32>::var_reader(format!("v_Axle_mps_{bogie}_{axle}"));

        let mg_relay = Shared::new(false);

        let traction_unit = three_phase_traction_unit(
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
                .set_source_voltage(Shared::new(1.0))
                .build(),
        );

        let voltage = Shared::new(1.0f32);

        mg_relay
            .delay_relay(0.14, 0.14)
            .to_float()
            .multiply(&voltage)
            .rail_brake(
                RailBrakeProperties::builder()
                    .reference_force(128_000.0)
                    .min_voltage(0.8)
                    .sound_pitch_base(0.8)
                    .sound_pitch_per_mps(0.05)
                    .bogie_index(bogie)
                    .variable_sound_volume(format!("Snd_Mg_{vehicle_part}_Friction_vol"))
                    .variable_sound_control(format!("Snd_Mg_{vehicle_part}"))
                    .variable_sound_pitch("Snd_Mg_Friction_pitch")
                    .build(),
            );
        traction_unit
            .wheel_force
            .var_writer(format!("M_Axle_N_{bogie}_{axle}"));
        traction_unit
            .wheel_force
            .var_writer(format!("Snd_Traction_{vehicle_part}"));
        TractionUnit {
            traction_unit,
            wheelspeed,
            mg_relay,
        }
    };

    let traction_units = [
        traction_unit(0, 1, "A".into()),
        traction_unit(1, 1, "C".into()),
        traction_unit(2, 0, "B".into()),
    ];

    state.sanding.sanding_unit(
        SandingUnitProperties::builder()
            .bogie_index(0_usize)
            .axle_index(1_usize)
            .sound_start("Snd_Sanden_Strt")
            .sound_loop("Snd_Sanden_Loop")
            .sound_stop("Snd_Sanden_Stop")
            .build(),
    );
    state.sanding.sanding_unit(
        SandingUnitProperties::builder()
            .bogie_index(1_usize)
            .axle_index(1_usize)
            .build(),
    );
    state.sanding.sanding_unit(
        SandingUnitProperties::builder()
            .bogie_index(2_usize)
            .axle_index(0_usize)
            .build(),
    );

    for traction_unit in traction_units.iter() {
        state.mg.clone().forward(&traction_unit.mg_relay);
    }

    let hydraulic_brake_target = Shared::new(0.0);
    let parking_brake_target = Shared::new(0.0);

    let add_brake_unit = |bogie: usize, axle: usize| {
        brake_combination(
            BrakeCombinationProperties::builder()
                .variable(format!("MBrake_Axle_N_{bogie}_{axle}"))
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

    add_brake_unit(0, 1);
    add_brake_unit(1, 1);
    add_brake_unit(2, 0);

    {
        let speed_shared = state.speed.clone();
        let richtungswender = state.direction.clone();
        let sollwertgeber = state.target.clone();
        let federspeicher = state.federspeicher.clone();

        spawn(async move {
            let mut mode_fixed = true;

            let mut prev_speed = 0.0;

            loop {
                let richtungswender = richtungswender.get();
                let sollwertgeber = sollwertgeber.get();

                let fast_brake = sollwertgeber < -0.95;
                let emergency_brake = false;

                let max_brake = fast_brake || emergency_brake;

                let schleuderschutz_active = false;
                let gleitschutz_active = false;

                let reversed = richtungswender == TractionDirection::Backward;

                // für den Gleitschutz wird das linke Rad am Drehgstell des mittleren Wagenteils benutzt
                let speed = get_var::<f32>("v_Axle_mps_1_0");
                speed_shared.set_only_on_change(speed);

                let speed_in_dir = if reversed { -speed } else { speed };

                // Traction ----------------------------------------------------

                let mut target_traction = if max_brake {
                    -1.0
                } else if sollwertgeber < 0.0 {
                    sollwertgeber * 1.111
                } else if (!reversed && speed_in_dir > VMAX)
                    || (reversed && speed_in_dir > VMAX_BACK)
                    || schleuderschutz_active
                {
                    0.0
                } else {
                    sollwertgeber
                };

                let mode = if target_traction > 0.01 {
                    if reversed {
                        TractionUnitMode::Backward
                    } else {
                        TractionUnitMode::Forward
                    }
                } else if target_traction < -0.01 {
                    TractionUnitMode::Brake
                } else {
                    TractionUnitMode::Off
                };

                traction_mode.set_only_on_change(mode);

                let mode_acceleration =
                    mode == TractionUnitMode::Forward || mode == TractionUnitMode::Backward;

                if (mode_acceleration && target_traction > 0.0) || speed_in_dir > 0.1 {
                    mode_fixed = false;
                } else if speed_in_dir < 0.1 && !mode_acceleration {
                    mode_fixed = true;
                }

                if gleitschutz_active && mode == TractionUnitMode::Brake {
                    target_traction /= 3.0;
                }

                target_force.set_only_on_change(target_traction.abs());

                // Parking brake ------------------------------------------------

                let federspeicher_active = federspeicher.get();

                parking_brake_target.set_only_on_change(if federspeicher_active {
                    1.0
                } else {
                    0.0
                });

                // Pneumatic Brake ----------------------------------------------------------------

                set_var(
                    "Snd_BrakeFlirr",
                    if mode == TractionUnitMode::Brake {
                        traction_units[0].traction_unit.wheel_force.get().abs() / MAXBRAKEFORCE_N
                    } else {
                        0.0
                    },
                );

                let mut pneu_target = if (mode_fixed && !federspeicher_active) || max_brake {
                    1.0
                } else if mode == TractionUnitMode::Brake {
                    target_traction.abs() * (1.0 - speed.abs() / V_EBRAKE_LIMIT).max(0.0)
                } else {
                    0.0
                };

                if gleitschutz_active {
                    pneu_target /= 3.0;
                }

                hydraulic_brake_target.set_only_on_change(pneu_target);

                // Additional sounds --------------------------------------------

                if speed == 0.0 && prev_speed != 0.0 {
                    set_var("Snd_Halteruck", true);
                }

                prev_speed = speed;

                // ----------------------------------------------------------------

                wait::next_tick().await;
            }
        });
    }

    state
}
