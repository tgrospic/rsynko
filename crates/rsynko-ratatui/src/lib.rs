#![doc = include_str!("../README.md")]

mod clock;
mod downloads;
mod inspections;
mod screen;
mod transfers;

/// States the work as a screen a reader moves around in.
pub mod terminal;

pub use rsynko_ui::Application;
