#[cfg(feature = "flame_it")]
#[macro_use]
extern crate flamer;

pub mod repository;
pub mod store;
pub mod timesheet;

pub use crate::repository::Repository;
pub use crate::store::{
    meta::Meta,
    patch::{Patch, PatchRef},
    Store,
};
pub use crate::timesheet::{Event, Timesheet};

pub type EventRef = String;
pub type Tag = String;
