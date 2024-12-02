use bevy::{prelude::BuildChildren, utils::all_tuples};

use crate::{ConstructContext, ConstructContextPatchExt, ConstructError, Patch};

/// Represents a tree of entities and patches to be applied to them.
pub struct EntityPatch<P: Patch, C: EntityPatchChildren> {
    // inherit: ?
    /// Patch that will be constructed and inserted as a bundle on this entity.
    pub patch: P,
    /// Zero or more [`EntityPatch`]es for the children of this entity.
    pub children: C,
}

/// A tuple of [`EntityPatch`]es spawnable as children to the entity in a [`ConstructContext`].
pub trait EntityPatchChildren {
    /// Recursively spawns all the children and their descendant patches.
    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError>;
}

impl EntityPatchChildren for () {
    fn spawn(self, _context: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }
}

impl<P: Patch, C: EntityPatchChildren> EntityPatchChildren for EntityPatch<P, C> {
    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        let id = context.world.spawn_empty().id();
        ConstructContext {
            id,
            world: context.world,
        }
        .spawn_entity_patch(self)?;
        context.world.entity_mut(context.id).add_child(id);
        Ok(())
    }
}

// Tuple impls
macro_rules! impl_entity_patch_children_tuple {
    ($(#[$meta:meta])* $(($P:ident, $C:ident, $e:ident)),*) => {
        $(#[$meta])*
        impl<$($P: Patch),*,$($C: EntityPatchChildren),*> EntityPatchChildren
            for ($(EntityPatch<$P, $C>,)*)
        {
            fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
                let ($($e,)*) = self;
                $($e.spawn(context)?;)*
                Ok(())
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_entity_patch_children_tuple,
    1,
    12,
    P,
    C,
    e
);

/// Extension trait implementing [`EntityPatch`] utilities for [`ConstructContext`].
pub trait ConstructContextEntityPatchExt {
    /// Spawns an [`EntityPatch`] recursively.
    fn spawn_entity_patch<P: Patch, C: EntityPatchChildren>(
        &mut self,
        entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError>;
}

impl<'a> ConstructContextEntityPatchExt for ConstructContext<'a> {
    fn spawn_entity_patch<P: Patch, C: EntityPatchChildren>(
        &mut self,
        mut entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError> {
        let bundle = self.construct_from_patch(&mut entity_patch.patch)?;
        self.world.entity_mut(self.id).insert(bundle);

        entity_patch.children.spawn(self)?;

        Ok(self)
    }
}
