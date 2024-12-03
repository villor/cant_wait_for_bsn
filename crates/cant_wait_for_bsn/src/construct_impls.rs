use bevy::prelude::*;
use bevy::utils::all_tuples;

use crate::{Construct, ConstructContext, ConstructError};

// Asset handles
impl<T: Asset> Construct for Handle<T> {
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
        Ok(context.world.resource::<AssetServer>().load(path))
    }
}

// Tuple impls
macro_rules! impl_construct_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: Construct),*> Construct for ($($T,)*)
        {
            type Props = ($($T::Props,)*);

            fn construct(
                _context: &mut ConstructContext,
                props: Self::Props,
            ) -> Result<Self, ConstructError> {
                let ($($t,)*) = props;
                $(let $t = $T::construct(_context, $t)?;)*
                Ok(($($t,)*))
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_construct_for_tuple,
    0,
    12,
    T,
    t
);

/// Entity reference constructable using [`EntityPath`], allowing passing either entity name or id as prop.
#[derive(Deref, Clone)]
pub struct ConstructEntity(Entity);

/// The construct prop for [`ConstructEntity`].
#[derive(Default, Clone)]
pub enum EntityPath {
    /// None
    #[default]
    None,
    /// Name
    Name(&'static str),
    /// Entity
    Entity(Entity),
}

impl From<&'static str> for EntityPath {
    fn from(value: &'static str) -> Self {
        Self::Name(value)
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

// TODO: This blanket impl is mentioned in the scene discussion, but is ambigous with custom impl Construct (for Default types), and tuple impls.
// impl<T: Default + Clone> Construct for T {
//     type Props = T;
//     #[inline]
//     fn construct(
//         _context: &mut ConstructContext,
//         props: Self::Props,
//     ) -> Result<Self, ConstructError> {
//         Ok(props)
//     }
// }

// Workaround for missing Default blanket, implement for some Bevy components to play with
macro_rules! impl_default_workaround {
    ($T:ident) => {
        impl Construct for $T {
            type Props = $T;
            #[inline]
            fn construct(
                _context: &mut ConstructContext,
                props: Self::Props,
            ) -> Result<Self, ConstructError> {
                Ok(props)
            }
        }
    };
}
impl_default_workaround!(Transform);
impl_default_workaround!(Node);
impl_default_workaround!(BackgroundColor);
impl_default_workaround!(BorderColor);
impl_default_workaround!(BorderRadius);
impl_default_workaround!(Text);
impl_default_workaround!(TextFont);
impl_default_workaround!(TextColor);
impl_default_workaround!(Camera2d);
impl_default_workaround!(Name);
