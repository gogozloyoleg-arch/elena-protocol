//! Механизм «Эхо-локации» для обнаружения двойной траты.

pub mod echo;

pub use echo::{CollisionDetector, EchoConfig, EchoLocator};
