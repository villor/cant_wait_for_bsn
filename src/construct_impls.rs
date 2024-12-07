use alloc::borrow::Cow;

use bevy::prelude::*;

use crate::{Construct, ConstructContext, ConstructError};

/// Constructable asset handle (because [`Handle<T>`] implements Default in Bevy right now)
#[derive(Deref, DerefMut, Clone, Reflect)]
pub struct ConstructHandle<T: Asset>(Handle<T>);

impl<T: Asset> From<Handle<T>> for ConstructHandle<T> {
    fn from(value: Handle<T>) -> Self {
        ConstructHandle(value)
    }
}

impl<T: Asset> From<ConstructHandle<T>> for Handle<T> {
    fn from(value: ConstructHandle<T>) -> Self {
        value.0
    }
}

impl<T: Asset> Construct for ConstructHandle<T> {
    //type Props = AssetPath<'static>;
    type Props = &'static str;

    fn construct(
        context: &mut ConstructContext,
        path: Self::Props,
    ) -> Result<Self, ConstructError> {
        // if let Err(err) = path.validate() {
        //     return Err(ConstructError::InvalidProps {
        //         message: format!("Invalid Asset Path: {err}").into(),
        //     });
        // }
        Ok(context.world.resource::<AssetServer>().load(path).into())
    }
}

/// Entity reference constructable using [`EntityPath`], allowing passing either entity name or id as prop.
#[derive(Deref, DerefMut, Clone, Reflect)]
pub struct ConstructEntity(Entity);

impl From<Entity> for ConstructEntity {
    fn from(value: Entity) -> Self {
        ConstructEntity(value)
    }
}

impl From<ConstructEntity> for Entity {
    fn from(value: ConstructEntity) -> Self {
        value.0
    }
}

/// The construct prop for [`ConstructEntity`].
#[derive(Default, Clone, Reflect)]
pub enum EntityPath {
    /// None
    #[default]
    None,
    /// Name
    Name(Cow<'static, str>),
    /// Entity
    Entity(Entity),
}

impl From<&'static str> for EntityPath {
    fn from(value: &'static str) -> Self {
        Self::Name(value.into())
    }
}

impl From<String> for EntityPath {
    fn from(value: String) -> Self {
        Self::Name(value.into())
    }
}

impl From<Entity> for EntityPath {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

impl Construct for ConstructEntity {
    type Props = EntityPath;

    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        match props {
            EntityPath::Name(name) => {
                let mut query = context.world.query::<(Entity, &Name)>();
                let entity = query
                    .iter(context.world)
                    .filter(|(_, q_name)| q_name.as_str() == name)
                    .map(|(entity, _)| ConstructEntity(entity))
                    .next();

                entity.ok_or_else(|| ConstructError::InvalidProps {
                    message: format!("entity with name {} does not exist", name).into(),
                })
            }
            EntityPath::Entity(entity) => Ok(ConstructEntity(entity)),
            _ => Err(ConstructError::InvalidProps {
                message: "no entity supplied".into(),
            }),
        }
    }
}
