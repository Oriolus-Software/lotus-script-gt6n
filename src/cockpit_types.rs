use lotus_rt_extra::cockpit_simple::{CockpitSoundAndVarSetState, StepSwitchPosition};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RichtungswenderState {
    #[default]
    O,
    I,
    V,
    R,
}

impl StepSwitchPosition for RichtungswenderState {
    fn next(&self) -> Self {
        match self {
            RichtungswenderState::O => RichtungswenderState::I,
            RichtungswenderState::I => RichtungswenderState::V,
            RichtungswenderState::V => RichtungswenderState::R,
            RichtungswenderState::R => RichtungswenderState::R,
        }
    }

    fn previous(&self) -> Self {
        match self {
            RichtungswenderState::O => RichtungswenderState::O,
            RichtungswenderState::I => RichtungswenderState::O,
            RichtungswenderState::V => RichtungswenderState::I,
            RichtungswenderState::R => RichtungswenderState::V,
        }
    }

    fn generate_with_this_sound(
        sound: &Option<String>,
        _: Self,
        _: Self,
    ) -> Vec<CockpitSoundAndVarSetState<Self, f32>> {
        vec![
            CockpitSoundAndVarSetState {
                input: RichtungswenderState::O,
                output: 0.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: RichtungswenderState::I,
                output: 29.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: RichtungswenderState::V,
                output: 58.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: RichtungswenderState::R,
                output: 135.0,
                sound: sound.clone(),
            },
        ]
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlinkerSwitch {
    Left,
    #[default]
    Off,
    Right,
}

impl StepSwitchPosition for BlinkerSwitch {
    fn next(&self) -> Self {
        match self {
            BlinkerSwitch::Left => BlinkerSwitch::Off,
            BlinkerSwitch::Off => BlinkerSwitch::Right,
            BlinkerSwitch::Right => BlinkerSwitch::Right,
        }
    }

    fn previous(&self) -> Self {
        match self {
            BlinkerSwitch::Left => BlinkerSwitch::Left,
            BlinkerSwitch::Off => BlinkerSwitch::Left,
            BlinkerSwitch::Right => BlinkerSwitch::Off,
        }
    }

    fn generate_with_this_sound(
        sound: &Option<String>,
        _: Self,
        _: Self,
    ) -> Vec<CockpitSoundAndVarSetState<Self, f32>> {
        vec![
            CockpitSoundAndVarSetState {
                input: BlinkerSwitch::Left,
                output: 0.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: BlinkerSwitch::Off,
                output: 1.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: BlinkerSwitch::Right,
                output: 2.0,
                sound: sound.clone(),
            },
        ]
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutsideLightSwitch {
    #[default]
    Off,
    Stand,
    Abblend,
    Fern,
}

impl StepSwitchPosition for OutsideLightSwitch {
    fn next(&self) -> Self {
        match self {
            OutsideLightSwitch::Off => OutsideLightSwitch::Stand,
            OutsideLightSwitch::Stand => OutsideLightSwitch::Abblend,
            OutsideLightSwitch::Abblend => OutsideLightSwitch::Fern,
            OutsideLightSwitch::Fern => OutsideLightSwitch::Fern,
        }
    }

    fn previous(&self) -> Self {
        match self {
            OutsideLightSwitch::Off => OutsideLightSwitch::Off,
            OutsideLightSwitch::Stand => OutsideLightSwitch::Off,
            OutsideLightSwitch::Abblend => OutsideLightSwitch::Stand,
            OutsideLightSwitch::Fern => OutsideLightSwitch::Abblend,
        }
    }

    fn generate_with_this_sound(
        sound: &Option<String>,
        _: Self,
        _: Self,
    ) -> Vec<CockpitSoundAndVarSetState<Self, f32>> {
        vec![
            CockpitSoundAndVarSetState {
                input: OutsideLightSwitch::Off,
                output: 0.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: OutsideLightSwitch::Stand,
                output: 1.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: OutsideLightSwitch::Abblend,
                output: 2.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: OutsideLightSwitch::Fern,
                output: 3.0,
                sound: sound.clone(),
            },
        ]
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoorSwitch {
    Tuer1,
    #[default]
    Closed,
    Released,
    Open,
}

impl StepSwitchPosition for DoorSwitch {
    fn next(&self) -> Self {
        match self {
            DoorSwitch::Tuer1 => DoorSwitch::Closed,
            DoorSwitch::Closed => DoorSwitch::Released,
            DoorSwitch::Released => DoorSwitch::Open,
            DoorSwitch::Open => DoorSwitch::Open,
        }
    }

    fn previous(&self) -> Self {
        match self {
            DoorSwitch::Tuer1 => DoorSwitch::Tuer1,
            DoorSwitch::Closed => DoorSwitch::Tuer1,
            DoorSwitch::Released => DoorSwitch::Closed,
            DoorSwitch::Open => DoorSwitch::Released,
        }
    }

    fn generate_with_this_sound(
        sound: &Option<String>,
        _: Self,
        _: Self,
    ) -> Vec<CockpitSoundAndVarSetState<Self, f32>> {
        vec![
            CockpitSoundAndVarSetState {
                input: DoorSwitch::Tuer1,
                output: -1.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: DoorSwitch::Closed,
                output: 0.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: DoorSwitch::Released,
                output: 1.0,
                sound: sound.clone(),
            },
            CockpitSoundAndVarSetState {
                input: DoorSwitch::Open,
                output: 2.0,
                sound: sound.clone(),
            },
        ]
    }
}
