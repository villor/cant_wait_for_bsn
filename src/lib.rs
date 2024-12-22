//! Naive, incomplete, and hacky implementation of the "Next generation scene proposal" for Bevy.

#![allow(internal_features)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(rustdoc_internals))]

extern crate alloc;

mod bsn_helpers;
mod bsn_reflect;
mod construct;
mod construct_impls;
mod construct_reflect;
mod dynamic;
mod entity_patch;
mod hot_patch;
mod hot_reload;
mod patch;

use bevy::app::App;
use bevy::app::Plugin;

pub use bsn_helpers::*;
pub use bsn_reflect::*;
pub use construct::*;
pub use construct_impls::*;
pub use construct_reflect::*;
pub use dynamic::*;
pub use entity_patch::*;
pub use hot_patch::*;
pub use hot_reload::*;
pub use patch::*;

//pub use cant_wait_for_bsn_macros::bsn;
pub use cant_wait_for_bsn_macros::bsn_hot as bsn; // TODO: Feature flag

pub use cant_wait_for_bsn_macros::Construct;

pub use cant_wait_for_bsn_parse as parse;

/// Registers all the necessary types for reflection-based dynamic scenes.
pub struct CantWaitForBsnPlugin;

impl Plugin for CantWaitForBsnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ConstructableTextFont>();
        register_reflect_construct(app);
        register_reflect_from_bsn(app);
    }
}
