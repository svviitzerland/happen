pub mod ecs;
pub mod app;

pub use ecs::{
    Component, Entity, World, Resource, Events, Event,
    SparseSet, ComponentStorage,
};
pub use app::{App, Plugin, SystemSchedule, Stage, Time};

pub const STAGE_FIRST: &str = "first";
pub const STAGE_PRE_UPDATE: &str = "pre_update";
pub const STAGE_UPDATE: &str = "update";
pub const STAGE_POST_UPDATE: &str = "post_update";
pub const STAGE_LAST: &str = "last";
