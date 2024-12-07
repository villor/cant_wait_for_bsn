use bevy::{
    prelude::{
        BuildChildren, ChildBuild, ChildBuilder, Commands, Entity, EntityCommand, EntityCommands,
        World,
    },
    utils::all_tuples,
};

use crate::{
    ConstructContext, ConstructContextPatchExt, ConstructError, DynamicPatch, DynamicScene, Patch,
};

/// Convenience trait for [`EntityPatch`].
pub trait Scene: Sized {
    /// Constructs a [`Scene`], inserts the components to the context entity, and recursively spawns scene descendants.
    fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Constructs and spawns a [`Scene`] as a child under the context entity recursively.
    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        let id = context.world.spawn_empty().id();
        context.world.entity_mut(context.id).add_child(id);

        self.construct(&mut ConstructContext {
            id,
            world: context.world,
        })?;

        Ok(())
    }

    /// Converts a static scene representation to dynamic by applying the dynamic patches to a [`DynamicScene`] and spawning their children.
    ///
    /// This is what powers [`EntityPatch`] inheritance.
    fn apply_dynamic(
        self,
        context: &mut ConstructContext,
        scene: &mut DynamicScene,
    ) -> Result<(), ConstructError>;
}

/// Zero or more [`Scene`]es forming a set of children or inherited patches. Implemented for tuples of [`Scene`].
pub trait SceneTuple {
    /// Whether this is an empty tuple
    const IS_EMPTY: bool;

    /// Recursively constructs/spawns all the entities in the tuple and their descendants under the context entity.
    fn spawn_children(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Applies each scene in the tuple to the dynamic scene by calling [`Scene::apply_dynamic`].
    fn apply_dynamic(
        self,
        context: &mut ConstructContext,
        scene: &mut DynamicScene,
    ) -> Result<(), ConstructError>;
}

impl SceneTuple for () {
    const IS_EMPTY: bool = true;

    fn spawn_children(self, _context: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }

    fn apply_dynamic(
        self,
        _: &mut ConstructContext,
        _: &mut DynamicScene,
    ) -> Result<(), ConstructError> {
        Ok(())
    }
}

// Tuple impls
macro_rules! impl_scene_tuple {
    ($(#[$meta:meta])* $(($S:ident, $s:ident)),*) => {
        $(#[$meta])*
        impl<$($S: Scene),*> SceneTuple for ($($S,)*)
        {
            const IS_EMPTY: bool = false;

            fn spawn_children(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
                let ($($s,)*) = self;
                $($s.spawn(context)?;)*
                Ok(())
            }

            fn apply_dynamic(
                self,
                context: &mut ConstructContext,
                scene: &mut DynamicScene,
            ) -> Result<(), ConstructError> {
                let ($($s,)*) = self;
                $($s.apply_dynamic(context, scene)?;)*
                Ok(())
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_scene_tuple,
    1,
    12,
    S,
    s
);

/// Represents a tree of entities and patches to be applied to them.
pub struct EntityPatch<I, P, C>
where
    I: SceneTuple,
    P: Patch + DynamicPatch,
    C: SceneTuple,
{
    /// Inherited scenes.
    pub inherit: I,
    /// Patch that will be constructed and inserted on this entity.
    pub patch: P,
    /// Child scenes of this entity.
    pub children: C,
}

impl<I, P, C> Scene for EntityPatch<I, P, C>
where
    I: SceneTuple,
    P: Patch + DynamicPatch,
    C: SceneTuple,
{
    /// Constructs an [`EntityPatch`], inserts the resulting bundle to the context entity, and recursively spawns children.
    fn construct(mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        if !I::IS_EMPTY {
            // Dynamic scene
            let mut dynamic_scene = DynamicScene::default();
            self.apply_dynamic(context, &mut dynamic_scene)?;
            dynamic_scene.construct(context)?;
            // TODO: Spawn children here instead?
        } else {
            // Static scene
            let bundle = context.construct_from_patch(&mut self.patch)?;
            context.world.entity_mut(context.id).insert(bundle);
            self.children.spawn_children(context)?;
        }

        Ok(())
    }

    fn apply_dynamic(
        mut self,
        context: &mut ConstructContext,
        scene: &mut DynamicScene,
    ) -> Result<(), ConstructError> {
        // Apply the inherited patches
        self.inherit.apply_dynamic(context, scene)?;

        // Apply this patch itself
        self.patch.dynamic_patch(scene);

        // Spawn this patches children
        // TODO: Move this so all children are spawned _after_ the entity is dynamically constructed?
        self.children.spawn_children(context)
    }
}

/// Extension trait implementing [`EntityPatch`] utilities for [`ConstructContext`].
pub trait ConstructContextEntityPatchExt {
    /// Constructs an [`EntityPatch`], inserts the components to the context entity, and recursively spawns the descendants.
    fn construct_entity_patch<I, P, C>(
        &mut self,
        entity_patch: EntityPatch<I, P, C>,
    ) -> Result<&mut Self, ConstructError>
    where
        I: SceneTuple,
        P: Patch + DynamicPatch,
        C: SceneTuple;

    /// Spawns an [`EntityPatch`] under the context entity recursively.
    fn spawn_entity_patch<I, P, C>(
        &mut self,
        entity_patch: EntityPatch<I, P, C>,
    ) -> Result<&mut Self, ConstructError>
    where
        I: SceneTuple,
        P: Patch + DynamicPatch,
        C: SceneTuple;
}

impl<'a> ConstructContextEntityPatchExt for ConstructContext<'a> {
    fn construct_entity_patch<I, P, C>(
        &mut self,
        entity_patch: EntityPatch<I, P, C>,
    ) -> Result<&mut Self, ConstructError>
    where
        I: SceneTuple,
        P: Patch + DynamicPatch,
        C: SceneTuple,
    {
        entity_patch.construct(self)?;
        Ok(self)
    }

    fn spawn_entity_patch<I, P, C>(
        &mut self,
        entity_patch: EntityPatch<I, P, C>,
    ) -> Result<&mut Self, ConstructError>
    where
        I: SceneTuple,
        P: Patch + DynamicPatch,
        C: SceneTuple,
    {
        entity_patch.spawn(self)?;
        Ok(self)
    }
}

/// Extension trait implementing [`EntityPatch`] utilities for [`EntityCommands`].
pub trait EntityCommandsEntityPatchExt {
    /// Constructs an [`EntityPatch`] and applies it to the entity.
    fn construct_patch<I, P, C>(&mut self, entity_patch: EntityPatch<I, P, C>) -> EntityCommands
    where
        I: SceneTuple + Send + 'static,
        P: Patch + DynamicPatch + Send + 'static,
        C: SceneTuple + Send + 'static;
}

struct ConstructEntityPatchCommand<I, P, C>(EntityPatch<I, P, C>)
where
    I: SceneTuple + Send + 'static,
    P: Patch + DynamicPatch + Send + 'static,
    C: SceneTuple + Send + 'static;

impl<I, P, C> EntityCommand for ConstructEntityPatchCommand<I, P, C>
where
    I: SceneTuple + Send + 'static,
    P: Patch + DynamicPatch + Send + 'static,
    C: SceneTuple + Send + 'static,
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
    fn construct_patch<
        I: SceneTuple + Send + 'static,
        P: Patch + DynamicPatch + Send + 'static,
        C: SceneTuple + Send + 'static,
    >(
        &mut self,
        entity_patch: EntityPatch<I, P, C>,
    ) -> EntityCommands {
        self.queue(ConstructEntityPatchCommand(entity_patch));
        self.reborrow()
    }
}

struct ConstructSceneCommand<S>(S)
where
    S: Scene + Send + 'static;

impl<S> EntityCommand for ConstructSceneCommand<S>
where
    S: Scene + Send + 'static,
{
    fn apply(self, id: Entity, world: &mut World) {
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
