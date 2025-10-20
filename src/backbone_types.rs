use lotus_rt_extra::{
    backbone::TypedMapKey,
    cockpit_simple::{ButtonInOutState, ButtonTwoSidedSpringLoadedState},
    observer::Observer,
};

use crate::cockpit_types::{
    BackDriveSwitch, BlinkerSwitch, DoorSwitch, OutsideLightSwitch, RichtungswenderState,
};

use strum::EnumIter;

#[derive(Hash, PartialEq, Eq, Default)]
pub enum CockpitSide {
    #[default]
    A,
    B,
}

impl From<CockpitSide> for usize {
    fn from(value: CockpitSide) -> Self {
        value as usize
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum SifaPosition {
    Sollwertgeber,
    Button,
}

#[derive(Hash, PartialEq, Eq)]
pub enum Gt6nCockpitInputBools {
    Sifa(SifaPosition),
    Sanden,
    MgBremse,
    Klingel(CockpitSide),
    Kinderwagen,
    Rollstuhl,
    BeleuchtungFahrgastraum,
    Schloss(CockpitSide),
    Tuer(CockpitSide, u8),
}

impl TypedMapKey for Gt6nCockpitInputBools {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq)]
pub enum Gt6nCockpitInputInts {
    BeleuchtungFahrerraum,
    Scheibenwischer,
    Zugbildung,
}

impl TypedMapKey for Gt6nCockpitInputInts {
    type Value = Observer<i8>;
}

#[derive(Hash, PartialEq, Eq)]
pub enum Gt6nCockpitInputFloats {
    Sollwertgeber,
}

impl TypedMapKey for Gt6nCockpitInputFloats {
    type Value = Observer<f32>;
}

#[derive(Hash, PartialEq, Eq)]
pub enum Gt6nCockpitInputTwoSidedSpringLoadedState {
    Pantograph,
    Hauptschalter,
    Sprechstelle,
}

impl TypedMapKey for Gt6nCockpitInputTwoSidedSpringLoadedState {
    type Value = Observer<ButtonTwoSidedSpringLoadedState>;
}

#[derive(Hash, PartialEq, Eq)]
pub enum Gt6nCockpitInputInOutState {
    FederspeicherOverwrite,
    Warnblinker,
}

impl TypedMapKey for Gt6nCockpitInputInOutState {
    type Value = Observer<ButtonInOutState>;
}

#[derive(Hash, PartialEq, Eq, EnumIter)]
pub enum Gt6nCockpitLeuchtmelder {
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

impl TypedMapKey for Gt6nCockpitLeuchtmelder {
    type Value = Observer<bool>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct Gt6nRichtungswender;

impl TypedMapKey for Gt6nRichtungswender {
    type Value = Observer<RichtungswenderState>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct Gt6nOutsideLightSwitch;

impl TypedMapKey for Gt6nOutsideLightSwitch {
    type Value = Observer<OutsideLightSwitch>;
}

#[derive(Hash, PartialEq, Eq)]
pub enum Gt6nBlinkerSwitch {
    Sw(CockpitSide),
}

impl TypedMapKey for Gt6nBlinkerSwitch {
    type Value = Observer<BlinkerSwitch>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct Gt6nDoorSwitch;

impl TypedMapKey for Gt6nDoorSwitch {
    type Value = Observer<DoorSwitch>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct Gt6nBackDriveSwitch;

impl TypedMapKey for Gt6nBackDriveSwitch {
    type Value = Observer<BackDriveSwitch>;
}
