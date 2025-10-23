use lotus_extra::types::CockpitSide;
use lotus_rt_extra::{
    backbone::TypedMapKey,
    cockpit_simple::{ButtonInOutState, ButtonTwoSidedSpringLoadedState},
    doors::DoorControlMode,
    observer::Observer,
    vehicle_systems::BlinkerState,
};

use crate::{cockpit_types, systems_interface};
use strum::EnumIter;

// ================================================================================
// General
// ================================================================================

#[derive(Hash, PartialEq, Eq)]
pub struct SystemActive;

impl TypedMapKey for SystemActive {
    type Value = Observer<bool>;
}

// #[derive(Hash, PartialEq, Eq)]
// pub struct TractionDirectionA;

// impl TypedMapKey for TractionDirectionA {
//     type Value = Observer<TractionDirection>;
// }

#[derive(Hash, PartialEq, Eq)]
pub struct VehicleSpeed;

impl TypedMapKey for VehicleSpeed {
    type Value = Observer<f32>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct Voltage;

impl TypedMapKey for Voltage {
    type Value = Observer<f32>;
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ActiveCockpit;

impl TypedMapKey for ActiveCockpit {
    type Value = Observer<systems_interface::ActiveCockpit>;
}

// ================================================================================
// Cockpit Inputs
// ================================================================================

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum SifaPosition {
    Sollwertgeber,
    Button,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CockpitInputBools {
    Sifa(SifaPosition),
    Sanden,
    MgBremse,
    Klingel(CockpitSide),
    Kinderwagen,
    Rollstuhl,
    BeleuchtungFahrgastraum,
    Schloss(CockpitSide),
    SchlossLock(CockpitSide),
    Tuer(CockpitSide, u8),
}

impl TypedMapKey for CockpitInputBools {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CockpitInputInts {
    BeleuchtungFahrerraum,
    Scheibenwischer,
    Zugbildung,
}

impl TypedMapKey for CockpitInputInts {
    type Value = Observer<i8>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CockpitInputFloats {
    Sollwertgeber,
}

impl TypedMapKey for CockpitInputFloats {
    type Value = Observer<f32>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CockpitInputTwoSidedSpringLoadedState {
    Pantograph,
    Hauptschalter,
    Sprechstelle,
}

impl TypedMapKey for CockpitInputTwoSidedSpringLoadedState {
    type Value = Observer<ButtonTwoSidedSpringLoadedState>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CockpitInputInOutState {
    FederspeicherOverwrite,
    Warnblinker,
}

impl TypedMapKey for CockpitInputInOutState {
    type Value = Observer<ButtonInOutState>;
}

#[derive(Hash, PartialEq, Eq, EnumIter, Debug)]
pub enum CockpitLeuchtmelder {
    Federspeicher,
    Fernlicht,
    BlinkerRechts(CockpitSide),
    BlinkerLinks(CockpitSide),
    Warnblinker,
    DoorsClosed,
    Haltewunsch,
    Kinderwagen,
    Rollstuhl,
    Schienenbremse,
    Sifa,
    Sprechstelle,
    Hauptschalter,
    Notstart,
    Notablegen,
}

impl TypedMapKey for CockpitLeuchtmelder {
    type Value = Observer<bool>;
}

//------------- special ------------

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct Richtungswender;

impl TypedMapKey for Richtungswender {
    type Value = Observer<cockpit_types::RichtungswenderState>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct OutsideLightSwitch;

impl TypedMapKey for OutsideLightSwitch {
    type Value = Observer<cockpit_types::OutsideLightSwitch>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum BlinkerSwitch {
    Sw(CockpitSide),
}

impl TypedMapKey for BlinkerSwitch {
    type Value = Observer<cockpit_types::BlinkerSwitch>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct DoorSwitch;

impl TypedMapKey for DoorSwitch {
    type Value = Observer<cockpit_types::DoorSwitch>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct BackDriveSwitch;

impl TypedMapKey for BackDriveSwitch {
    type Value = Observer<cockpit_types::BackDriveSwitch>;
}

// ================================================================================
// Lights
// ================================================================================

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum Lights {
    Voltage,
    Fahrgastraum,
    Stand,
    Abblend,
    Fern,
    Rueck,
    Rueckfahr,
    Brems,
    BlinkerLampeRechts,
    BlinkerLampeLinks,
    LmWarnblinker,
    CockpitMain,
    CockpitBegleiter,
    Instrumente,
}

impl TypedMapKey for Lights {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct LightBlinkerState;

impl TypedMapKey for LightBlinkerState {
    type Value = Observer<BlinkerState>;
}

// ================================================================================
// Doors Inputs
// ================================================================================

#[derive(Hash, PartialEq, Eq)]
pub struct DoorsReleased;

impl TypedMapKey for DoorsReleased {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct Door1Force;

impl TypedMapKey for Door1Force {
    type Value = Observer<DoorControlMode>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum DoorRequest {
    DoorLeft(i8),
    DoorRight(i8),
}

impl TypedMapKey for DoorRequest {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct OverrideNoWarning;

impl TypedMapKey for OverrideNoWarning {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct DoorsAllClosed;

impl TypedMapKey for DoorsAllClosed {
    type Value = Observer<bool>;
}

// ================================================================================
// Passenger Elements
// ================================================================================

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum PassengerDoorButtons {
    DoorRight(i8),
    DoorLeft(i8),
}

impl TypedMapKey for PassengerDoorButtons {
    type Value = Observer<bool>;
}

// ================================================================================
// Misc
// ================================================================================

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum MiscBools {
    Klingel,
}

impl TypedMapKey for MiscBools {
    type Value = Observer<bool>;
}
