use bevy::{prelude::BuildChildren, utils::all_tuples};

use crate::{ConstructContext, ConstructContextPatchExt, ConstructError, Patch};

pub struct EntityPatch<P: Patch, C: EntityPatchChildren> {
    // inherit: ?
    pub patch: P,
    pub children: C,
}

pub trait EntityPatchChildren {
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
    ($(($P:ident, $C:ident, $e:ident)),*) => {
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

all_tuples!(impl_entity_patch_children_tuple, 1, 12, P, C, e);

pub trait ConstructContextEntityPatchExt {
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
