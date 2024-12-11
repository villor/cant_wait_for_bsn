use core::any::TypeId;

use bevy::{
    prelude::{BuildChildren, Component},
    reflect::{PartialReflect, Reflect},
    utils::{all_tuples, TypeIdMap},
};

use crate::{Construct, ConstructContext, ConstructError, ConstructPatch, PatchProps};

/// Dynamic patch
pub trait DynamicPatch: Send + Sync + 'static {
    /// Adds this patch "on top" of the dynamic scene by updating the dynamic props.
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);
}

// Tuple impls
macro_rules! impl_patch_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: DynamicPatch),*> DynamicPatch for ($($T,)*) {
            fn dynamic_patch(&mut self, _scene: &mut DynamicScene) {
                let ($($t,)*) = self;
                $($t.dynamic_patch(_scene);)*
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_patch_for_tuple,
    0,
    12,
    T,
    t
);

impl<C, F, P> DynamicPatch for ConstructPatch<C, F>
where
    C: Construct<Props = P> + Component + PartialReflect + Sync + Send + 'static,
    P: Reflect + Default + Clone + Sync + Send + 'static,
    F: Fn(&mut C::Props) + Clone + Sync + Send + 'static,
{
    fn dynamic_patch(&mut self, scene: &mut DynamicScene) {
        let dynamic_props = scene
            .component_props
            .entry(TypeId::of::<C>())
            .or_insert_with(|| DynamicProps {
                construct: &|context, dynamic_props| {
                    // Kind of hacky to do this here, but it'll do for now
                    let props = {
                        let entity = context.world.entity_mut(context.id);
                        let mut props = entity.get::<PatchProps<C>>().cloned().unwrap_or_default();

                        for patch in dynamic_props.patches.iter() {
                            (patch)(props.props.as_reflect_mut());
                        }

                        props.clone()
                    };

                    let component = context.construct::<C>(props.props)?;

                    let mut entity = context.world.entity_mut(context.id);
                    entity.insert(component);

                    Ok(())
                },
                patches: Vec::new(),
            });

        let func = self.func.clone();

        dynamic_props
            .patches
            .push(Box::new(move |props: &mut dyn Reflect| {
                (func)(props.downcast_mut::<C::Props>().unwrap());
            }));
    }
}

struct DynamicProps<'a> {
    patches: Vec<Box<dyn Fn(&mut dyn Reflect)>>,
    construct: &'a dyn Fn(&mut ConstructContext, &DynamicProps<'a>) -> Result<(), ConstructError>,
}

/// A dynamic scene containing dynamic patches and children.
#[derive(Default)]
pub struct DynamicScene<'a> {
    component_props: TypeIdMap<DynamicProps<'a>>,
    children: Vec<DynamicScene<'a>>,
}

impl<'a> DynamicScene<'a> {
    /// Constructs the dynamic patches in the scene, inserts the resulting components, and spawns children recursively.
    pub fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        // Construct and hot patch components
        for (_, props) in self.component_props {
            (props.construct)(context, &props)?;
        }

        // Spawn children
        for child in self.children {
            let child_id = context.world.spawn_empty().id();
            context.world.entity_mut(context.id).add_child(child_id);
            child.construct(&mut ConstructContext {
                id: child_id,
                world: context.world,
            })?;
        }

        Ok(())
    }

    /// Add a child to the dynamic scene.
    pub fn push_child(&mut self, child: DynamicScene<'a>) {
        self.children.push(child);
    }
}
