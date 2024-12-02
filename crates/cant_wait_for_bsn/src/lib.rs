//! `cant_wait_for_bsn`

#![allow(internal_features)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(rustdoc_internals))]

extern crate alloc;

mod construct;
mod entity_patch;
mod patch;

pub use construct::*;
pub use entity_patch::*;
pub use patch::*;

pub use cant_wait_for_bsn_macros::bsn;
