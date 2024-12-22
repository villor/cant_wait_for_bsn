use core::any::TypeId;

use bevy::{
    log::{error, warn}, prelude::{AppTypeRegistry, BuildChildren, Component, Mut, ReflectComponent}, reflect::{PartialReflect, Reflect}, utils::{all_tuples, TypeIdMap}
};

use crate::{Construct, ConstructContext, ConstructError, ConstructPatch, ReflectConstruct};

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
        let patches = scene.component_props.entry(TypeId::of::<C>()).or_default();

        let func = self.func.clone();

        patches.push(Box::new(move |props: &mut dyn Reflect| {
            (func)(props.downcast_mut::<C::Props>().unwrap());
        }));
    }
}

// TODO: Better solution? Do we really need ReflectPatch?

/// Trait implemented for functions that can patch [`Reflect`] props.
pub trait ReflectPatch: Sync + Send {
    /// Patch the given props.
    fn patch(&self, props: &mut dyn Reflect);
}

impl<F> ReflectPatch for F
where
    F: Fn(&mut dyn Reflect) + Sync + Send,
{
    fn patch(&self, props: &mut dyn Reflect) {
        (self)(props);
    }
}

/// A dynamic scene containing dynamic patches and children.
#[derive(Default)]
pub struct DynamicScene {
    /// Maps component type ids to patches to be applied on the props before construction.
    pub component_props: TypeIdMap<Vec<Box<dyn ReflectPatch>>>,
    /// Children of the scene.
    pub children: Vec<DynamicScene>,
}

impl DynamicScene {
    /// Constructs the dynamic patches in the scene, inserts the resulting components, and spawns children recursively.
    pub fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        // Construct components
        for (type_id, patches) in self.component_props {
            context
                .world
                .resource_scope(|world, app_registry: Mut<AppTypeRegistry>| {
                    let registry = app_registry.read();
                    let t = registry
                        .get(type_id)
                        .expect("failed to get type from registry");
                    let Some(reflect_construct) = t.data::<ReflectConstruct>() else {
                        warn!(
                            "No registered ReflectConstruct for component: {:?}. Skipping construction. Consider adding #[reflect(Construct)].",
                            t.type_info().type_path()
                        );
                        return;
                    };
                    let Some(reflect_component) = t.data::<ReflectComponent>() else {
                        warn!(
                            "No registered ReflectComponent for component: {:?}. Skipping construction. Consider adding #[reflect(Component)].",
                            t.type_info().type_path()
                        );
                        return;
                    };

                    if reflect_construct.props_type_id == type_id {
                        // This is a Default + Clone construct, meaning it does not need construction and can be patched directly.
                        if !reflect_component.contains(world.entity(context.id)) {
                            let mut entity = world.entity_mut(context.id);
                            reflect_component.insert(&mut entity, reflect_construct.default_props().as_partial_reflect(), &registry);
                        }

                        let entity = world.entity_mut(context.id);
                        let mut component = reflect_component.reflect_mut(entity).expect("component should exist");
                        
                        for patch in patches.iter() {
                            patch.patch(component.as_reflect_mut());
                        }

                        return;
                    }

                    // Prepare props
                    let mut props = reflect_construct.default_props();
                    for patch in patches.iter() {
                        patch.patch(props.as_mut());
                    }

                    // Construct component
                    let Ok(component) = reflect_construct.construct(
                        &mut ConstructContext {
                            id: context.id,
                            world,
                        },
                        props,
                    ) else {
                        error!(
                            "failed to construct component: {:?}.",
                            t.type_info().type_path()
                        );
                        return;
                    };

                    // Insert component on entity
                    // TODO: Partial/hot patch for non Default + Clone constructs?
                    let mut entity = world.entity_mut(context.id);
                    reflect_component.apply_or_insert(&mut entity, component.as_ref(), &registry);
                });
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
    pub fn push_child(&mut self, child: DynamicScene) {
        self.children.push(child);
    }
}
