use bevy::prelude::Component;

use slabmap::{SlabMap, SlabMapId};

use crate::units::{ScreenVector, WorldPoint};

#[derive(Debug, Clone)]
pub enum Signal {
    /// Simple on/off signal
    Boolean(bool),
    /// Variable scalar value signal
    Scalar(f64),
    /// Screen vector signal, usually used for on-screen controls
    ScreenVector(ScreenVector),
    /// World position signal, useful for stuff like enemy targeting
    WorldPosition(WorldPoint),
}

impl Default for Signal {
    fn default() -> Self {
        Signal::Boolean(false)
    }
}

impl Signal {
    pub fn as_bool(&self) -> bool {
        match self {
            Signal::Boolean(bool) => *bool,
            Signal::Scalar(value) => *value > 0.0, // Any positive scalar value is true
            Signal::ScreenVector(vec) => vec.x != 0.0 || vec.y != 0.0, // Any non-zero vector is true
            Signal::WorldPosition(_) => true,                          // Any world point is true
        }
    }
}

type SignalTag = String;
pub type SignalId = SlabMapId<Signal>;

#[derive(Debug, Clone, Default, Component)]
pub struct Signals(pub SlabMap<SignalTag, Signal>);
