use core::any::TypeId;

use bevy::{
    prelude::{AppTypeRegistry, Component, Mut, ReflectComponent},
    reflect::{PartialReflect, Reflect},
    utils::{all_tuples, TypeIdMap},
};

use crate::{
    Construct, ConstructContext, ConstructError, ConstructPatch, Patch, 
};

/// Dynamic patch
pub trait DynamicPatch: Send + Sync + 'static {
    /// Adds this patch "on top" of the dynamic scene by updating the dynamic patches and (TODO) pushing any dynamic children.
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
    F: FnMut(&mut C::Props) + Sync + Send + 'static,
{
    fn dynamic_patch(&mut self, scene: &mut DynamicScene) {
        let dynamic_props = scene
            .component_props
            .entry(TypeId::of::<C>())
            .or_insert_with(|| DynamicProps {
                construct: Box::new(|context, props| {
                    Ok(Box::new(C::construct(context, props.take().unwrap())?))
                }),
                props: Box::new(P::default()),
            });

        let p = dynamic_props.props.downcast_mut::<P>().unwrap();
        self.patch(p);
    }
}

struct DynamicProps {
    construct: Box<dyn Fn(&mut ConstructContext, Box<dyn Reflect>) -> Result<Box<dyn PartialReflect>, ConstructError>>,
    props: Box<dyn Reflect>,
}

/// A dynamic scene containing dynamic patches.
#[derive(Default)]
pub struct DynamicScene {
    component_props: TypeIdMap<DynamicProps>,
    // children: Vec<DynamicScene>, // TODO: Children are static for now, dynamic children should only be needed for asset loaded scenes?
}

impl DynamicScene {
    /// Constructs the dynamic patches in the scene and inserts the resulting components onto the context entity.
    ///
    /// TODO: Also spawn the children?
    pub fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        let id = context.id;
        context
            .world
            .resource_scope(|world, type_registry: Mut<AppTypeRegistry>| {
                let type_registry = type_registry.read();

                for (type_id, props) in self.component_props {
                    let Some(type_registration) = type_registry.get(type_id) else {
                        bevy::log::warn!("Component type `{:?}` not found in type registry during DynamicScene construction, skipped.", type_id);
                        continue;
                    };
        
                    let Some(reflect_component) = type_registration.data::<ReflectComponent>() else {
                        bevy::log::warn!(
                            "Component `{:?}` is not reflectable, skipped.",
                            type_registration.type_info().type_path()
                        );
                        continue;
                    };
        
                    let component = (props.construct)(&mut ConstructContext {
                        id,
                        world
                    }, props.props)?;

                    let mut entity = world.entity_mut(id);
                    reflect_component.insert(&mut entity, component.as_ref(), &type_registry);
                }

                Ok(())
            })
    }
}
