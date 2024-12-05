use bevy::{prelude::Bundle, utils::TypeIdMap};
use downcast_rs::{impl_downcast, Downcast};

use crate::{Construct, ConstructContext, ConstructError, ConstructPatch};

// TODO: (maybe)

pub trait DynamicPatch: Downcast + Send + Sync + 'static {
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);
}
impl_downcast!(DynamicPatch);

#[derive(Default)]
pub struct DynamicScene {
    component_props: TypeIdMap<Box<dyn DynamicPatch>>,
    children: Vec<DynamicScene>,
}
