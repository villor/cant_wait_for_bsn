use alloc::borrow::Cow;

use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
    text::FontSmoothing,
};

use crate::{Construct, ConstructContext, ConstructError, ConstructProp};

/// Constructable asset handle (because [`Handle<T>`] implements Default in Bevy right now)
#[derive(Deref, DerefMut, Clone, Reflect, Debug)]
pub struct ConstructHandle<T: Asset>(pub Handle<T>);

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
#[derive(Deref, DerefMut, Debug, Clone, Reflect)]
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
#[derive(Default, Debug, Clone, Reflect)]
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

/// Constructable text font. Workaround for default-implmented [`TextFont`] in Bevy.
#[derive(Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ConstructableTextFont {
    /// Font
    pub font: ConstructHandle<Font>,
    /// Font size
    pub font_size: f32,
    /// Font smoothing
    pub font_smoothing: FontSmoothing,
}

#[allow(missing_docs)]
#[derive(Clone, Reflect)]
pub struct ConstructableTextFontProps {
    pub font: ConstructProp<ConstructHandle<Font>>,
    pub font_size: f32,
    pub font_smoothing: FontSmoothing,
}

impl Default for ConstructableTextFontProps {
    fn default() -> Self {
        let TextFont {
            font,
            font_size,
            font_smoothing,
        } = TextFont::default();
        Self {
            font: ConstructProp::Value(font.into()),
            font_size,
            font_smoothing,
        }
    }
}

impl Construct for ConstructableTextFont {
    type Props = ConstructableTextFontProps;
    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        Ok(Self {
            font: props.font.construct(context)?,
            font_size: props.font_size,
            font_smoothing: props.font_smoothing,
        })
    }
}

impl Component for ConstructableTextFont {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _component_id| {
            let constructable = world.get::<ConstructableTextFont>(entity).unwrap().clone();
            world.commands().entity(entity).insert(TextFont {
                font: constructable.font.into(),
                font_size: constructable.font_size,
                font_smoothing: constructable.font_smoothing,
            });
        });
        hooks.on_remove(|mut world, entity, _component_id| {
            if let Some(mut entity) = world.commands().get_entity(entity) {
                entity.remove::<TextFont>();
            }
        });
    }
}
