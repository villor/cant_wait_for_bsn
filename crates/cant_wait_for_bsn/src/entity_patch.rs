use bevy::{
    prelude::{
        BuildChildren, ChildBuild, ChildBuilder, Commands, Entity, EntityCommand, EntityCommands,
        World,
    },
    utils::all_tuples,
};

use crate::{ConstructContext, ConstructContextPatchExt, ConstructError, Patch};

// TODO: EntityPatch/Scene should be consumed when constructed/spawned, no?

/// Represents a tree of entities and patches to be applied to them.
pub struct EntityPatch<P: Patch, C: EntityPatchChildren> {
    // Inherited patches
    // pub inherit: I,
    /// Patch that will be constructed and inserted as a bundle on this entity.
    pub patch: P,
    /// Zero or more [`EntityPatch`]es for the children of this entity.
    pub children: C,
}

impl<P: Patch, C: EntityPatchChildren> EntityPatch<P, C> {
    /// Constructs an [`EntityPatch`], inserts the components to the context entity, and recursively spawns children.
    fn construct(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        // TODO: Patches of the same type has to be combined before constructing
        // Maybe dynamics is the way after all...
        let bundle = context.construct_from_patch(&mut self.patch)?;
        context.world.entity_mut(context.id).insert(bundle);

        self.children.spawn(context)?;

        Ok(())
    }

    /// Constructs and spawns an [`EntityPatch`] as a child under the context entity recursively.
    fn spawn(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        let id = context.world.spawn_empty().id();
        context.world.entity_mut(context.id).add_child(id);

        self.construct(&mut ConstructContext {
            id,
            world: context.world,
        })?;

        Ok(())
    }
}

/// Zero or more [`EntityPatch`]es forming a set of children. Implemented for tuples of [`EntityPatch`].
pub trait EntityPatchChildren {
    /// Recursively constructs/spawns all the children and their descendant patches.
    fn spawn(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError>;
}

impl EntityPatchChildren for () {
    fn spawn(&mut self, _context: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }
}

impl<P: Patch, C: EntityPatchChildren> EntityPatchChildren for EntityPatch<P, C> {
    fn spawn(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        self.spawn(context)
    }
}

// Tuple impls
macro_rules! impl_entity_patch_children_tuple {
    ($(#[$meta:meta])* $(($P:ident, $C:ident, $e:ident)),*) => {
        $(#[$meta])*
        impl<$($P: Patch),*, $($C: EntityPatchChildren),*> EntityPatchChildren
            for ($(EntityPatch<$P, $C>,)*)
        {
            fn spawn(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
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
    /// Constructs an [`EntityPatch`], inserts the components to the context entity, and recursively spawns the descendants.
    fn construct_entity_patch<P: Patch, C: EntityPatchChildren>(
        &mut self,
        entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError>;

    /// Spawns an [`EntityPatch`] under the context entity recursively.
    fn spawn_entity_patch<P: Patch, C: EntityPatchChildren>(
        &mut self,
        entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError>;
}

impl<'a> ConstructContextEntityPatchExt for ConstructContext<'a> {
    fn construct_entity_patch<P: Patch, C: EntityPatchChildren>(
        &mut self,
        mut entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError> {
        entity_patch.construct(self)?;
        Ok(self)
    }

    fn spawn_entity_patch<P: Patch, C: EntityPatchChildren>(
        &mut self,
        mut entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError> {
        entity_patch.spawn(self)?;
        Ok(self)
    }
}

/// Extension trait implementing [`EntityPatch`] utilities for [`EntityCommands`].
pub trait EntityCommandsEntityPatchExt {
    /// Constructs an [`EntityPatch`] and applies it to the entity.
    fn construct_patch<P, C>(&mut self, entity_patch: EntityPatch<P, C>) -> EntityCommands
    where
        P: Patch + Send + 'static,
        C: EntityPatchChildren + Send + 'static;
}

struct ConstructEntityPatchCommand<P, C>(EntityPatch<P, C>)
where
    P: Patch + Send + 'static,
    C: EntityPatchChildren + Send + 'static;

impl<P, C> EntityCommand for ConstructEntityPatchCommand<P, C>
where
    P: Patch + Send + 'static,
    C: EntityPatchChildren + Send + 'static,
{
    fn apply(self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
        context
            .construct_entity_patch(self.0)
            .expect("TODO failed to spawn_entity_patch in ConstructEntityPatchCommand");
    }
}

impl<'w> EntityCommandsEntityPatchExt for EntityCommands<'w> {
    // type Out = EntityCommands;
    fn construct_patch<P: Patch + Send + 'static, C: EntityPatchChildren + Send + 'static>(
        &mut self,
        entity_patch: EntityPatch<P, C>,
    ) -> EntityCommands {
        self.queue(ConstructEntityPatchCommand(entity_patch));
        self.reborrow()
    }
}

/// Convenience trait for [`EntityPatch`].
pub trait Scene {
    /// Constructs a [`Scene`], inserts the components to the context entity, and recursively spawns scene descendants.
    fn construct(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Constructs and spawns a [`Scene`] as a child under the context entity recursively.
    fn spawn(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Unpacks the patch and children of the scene, to use for inheritance.
    fn unpack(self) -> (impl Patch, impl EntityPatchChildren);
}

impl<P: Patch, C: EntityPatchChildren> Scene for EntityPatch<P, C> {
    fn construct(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        self.construct(context)
    }

    fn spawn(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        self.spawn(context)
    }

    fn unpack(self) -> (impl Patch, impl EntityPatchChildren) {
        (self.patch, self.children)
    }
}

struct ConstructSceneCommand<S>(S)
where
    S: Scene + Send + 'static;

impl<S> EntityCommand for ConstructSceneCommand<S>
where
    S: Scene + Send + 'static,
{
    fn apply(mut self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
        self.0
            .construct(&mut context)
            .expect("TODO failed to spawn_scene in ConstructSceneCommand");
    }
}

/// Scene spawning extension.
pub trait SpawnSceneExt {
    /// Spawn the given [`Scene`].
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> &mut Self;
}

impl<'w> SpawnSceneExt for Commands<'w, '_> {
    /// Spawn the given [`Scene`].
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> &mut Self {
        self.spawn_empty().queue(ConstructSceneCommand(scene));
        self
    }
}

impl<'w> SpawnSceneExt for ChildBuilder<'w> {
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> &mut Self {
        self.spawn_empty().queue(ConstructSceneCommand(scene));
        self
    }
}
