use bevy::{
    prelude::{
        BuildChildren, ChildBuild, ChildBuilder, Commands, Entity, EntityCommand, EntityCommands,
        World,
    },
    utils::all_tuples,
};

use crate::{ConstructContext, ConstructContextPatchExt, ConstructError, Patch};

/// Represents a tree of entities and patches to be applied to them.
pub struct EntityPatch<P: Patch, C: Scene> {
    // inherit: ?
    /// Patch that will be constructed and inserted as a bundle on this entity.
    pub patch: P,
    /// Zero or more [`EntityPatch`]es for the children of this entity.
    pub children: C,
}

/// One or more [`EntityPatch`]es forming a scene.
pub trait Scene {
    /// Recursively spawns all the children and their descendant patches.
    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError>;
}

impl Scene for () {
    fn spawn(self, _context: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }
}

impl<P: Patch, C: Scene> Scene for EntityPatch<P, C> {
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
        impl<$($P: Patch),*,$($C: Scene),*> Scene
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
    fn spawn_entity_patch<P: Patch, C: Scene>(
        &mut self,
        entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError>;
}

impl<'a> ConstructContextEntityPatchExt for ConstructContext<'a> {
    fn spawn_entity_patch<P: Patch, C: Scene>(
        &mut self,
        mut entity_patch: EntityPatch<P, C>,
    ) -> Result<&mut Self, ConstructError> {
        let bundle = self.construct_from_patch(&mut entity_patch.patch)?;
        self.world.entity_mut(self.id).insert(bundle);

        entity_patch.children.spawn(self)?;

        Ok(self)
    }
}

/// Extension trait implementing [`EntityPatch`] utilities for [`EntityCommands`].
pub trait EntityCommandsEntityPatchExt {
    /// Constructs an [`EntityPatch`] and applies it to the entity.
    fn construct_patch<P, C>(
        &mut self,
        entity_patch: impl Into<EntityPatch<P, C>>,
    ) -> EntityCommands
    where
        P: Patch + Send + 'static,
        C: Scene + Send + 'static;
}

struct ConstructEntityPatchCommand<P, C>(EntityPatch<P, C>)
where
    P: Patch + Send + 'static,
    C: Scene + Send + 'static;

impl<P, C> EntityCommand for ConstructEntityPatchCommand<P, C>
where
    P: Patch + Send + 'static,
    C: Scene + Send + 'static,
{
    fn apply(self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
        context
            .spawn_entity_patch(self.0)
            .expect("TODO failed to spawn_entity_patch in ConstructEntityPatchCommand");
    }
}

impl<'w> EntityCommandsEntityPatchExt for EntityCommands<'w> {
    // type Out = EntityCommands;
    fn construct_patch<P: Patch + Send + 'static, C: Scene + Send + 'static>(
        &mut self,
        entity_patch: impl Into<EntityPatch<P, C>>,
    ) -> EntityCommands {
        self.queue(ConstructEntityPatchCommand(entity_patch.into()));
        self.reborrow()
    }
}

struct SpawnSceneCommand<S>(S)
where
    S: Scene + Send + 'static;

impl<S> EntityCommand for SpawnSceneCommand<S>
where
    S: Scene + Send + 'static,
{
    fn apply(self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
        self.0
            .spawn(&mut context)
            .expect("TODO failed to spawn_scene in SpawnSceneCommand");
    }
}

/// Scene spawning extension.
pub trait SpawnSceneExt {
    /// Spawn the given [`Scene`].
    fn spawn_scene<S>(self, scene: S) -> Self
    where
        S: Scene + Send + 'static;
}

impl<'w> SpawnSceneExt for Commands<'w, '_> {
    fn spawn_scene<S>(mut self, scene: S) -> Self
    where
        S: Scene + Send + 'static,
    {
        let mut s = self.spawn_empty();
        s.queue(SpawnSceneCommand::<S>(scene));
        self
    }
}

impl<'w> SpawnSceneExt for ChildBuilder<'w> {
    fn spawn_scene<S>(mut self, scene: S) -> ChildBuilder<'w>
    where
        S: Scene + Send + 'static,
    {
        let mut s = self.spawn_empty();
        s.queue(SpawnSceneCommand::<S>(scene));
        self
    }
}
