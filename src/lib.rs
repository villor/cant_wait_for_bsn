//! Naive, incomplete, and hacky implementation of the "Next generation scene proposal" for Bevy.

#![allow(internal_features)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(rustdoc_internals))]

extern crate alloc;

mod bsn_helpers;
mod construct;
mod construct_impls;
mod dynamic;
mod entity_patch;
mod hot_patch;
mod patch;

pub use bsn_helpers::*;
pub use construct::*;
pub use construct_impls::*;
pub use dynamic::*;
pub use entity_patch::*;
pub use hot_patch::*;
pub use patch::*;

pub use cant_wait_for_bsn_macros::bsn;
pub use cant_wait_for_bsn_macros::Construct;
