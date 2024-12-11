use core::marker::PhantomData;

use bevy::utils::all_tuples;
use bevy::{ecs::reflect::ReflectComponent, prelude::Component, reflect::Reflect};

use crate::{Construct, ConstructContext, ConstructError, ConstructPatch, Patch};

/// Retained props to allow hot patching.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PatchProps<C>
where
    C: Construct + Component,
{
    pub(crate) props: <C as Construct>::Props,
    _marker: PhantomData<C>,
}

impl<C: Construct + Component> Clone for PatchProps<C> {
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            _marker: self._marker,
        }
    }
}

impl<C: Construct + Component> Default for PatchProps<C> {
    fn default() -> Self {
        Self {
            props: <C as Construct>::Props::default(),
            _marker: PhantomData,
        }
    }
}

/// Hot patch 🌶️
pub trait HotPatch {
    /// Hot patch the component on the entity directly.
    fn hot_patch(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError>;
}

// Tuple impls
macro_rules! impl_hot_patch_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: HotPatch),*> HotPatch for ($($T,)*) {
            fn hot_patch(&mut self, _context: &mut ConstructContext) -> Result<(), ConstructError> {
                let ($($t,)*) = self;
                $($t.hot_patch(_context)?;)*
                Ok(())
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_hot_patch_for_tuple,
    0,
    12,
    T,
    t
);

impl<C, F, P> HotPatch for ConstructPatch<C, F>
where
    C: Construct<Props = P> + Component,
    P: Default + Clone + Sync + Send + 'static,
    F: Fn(&mut C::Props) + Clone + Sync + Send + 'static,
{
    fn hot_patch(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        let props = {
            let mut entity = context.world.entity_mut(context.id);
            let mut props = entity.entry::<PatchProps<C>>().or_default();
            self.patch(&mut props.props);
            props.clone()
        };

        let component = context.construct::<C>(props.props)?;

        let mut entity = context.world.entity_mut(context.id);
        entity.insert(component);

        Ok(())
    }
}
