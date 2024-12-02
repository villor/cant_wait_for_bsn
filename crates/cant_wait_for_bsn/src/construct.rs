//! Based on implementation from [bevy_asky](https://github.com/shanecelis/bevy_asky/blob/main/src/construct.rs).
use alloc::borrow::Cow;
use bevy::utils::all_tuples;
use bevy::{ecs::system::EntityCommands, prelude::*};
use thiserror::Error;

/// Construction error
#[derive(Error, Debug)]
pub enum ConstructError {
    /// Invalid properties
    #[error("invalid properties {message:?}")]
    InvalidProps {
        /// Message
        message: Cow<'static, str>,
    },
    /// Missing resource
    #[error("missing resource {message:?}")]
    MissingResource {
        /// Message
        message: Cow<'static, str>,
    },
}

/// Construct property
#[derive(Clone)]
pub enum ConstructProp<T: Construct> {
    /// Direct Value
    Value(T),
    /// Properties
    Prop(T::Props),
}

impl<T: Construct> ConstructProp<T> {
    /// Consumes the [`ConstructProp`] and returns the inner value, constructed if necessary.
    ///
    /// Not part of scene discussion, but added for convenience.
    pub fn construct(self, context: &mut ConstructContext) -> Result<T, ConstructError> {
        match self {
            ConstructProp::Prop(p) => Construct::construct(context, p),
            ConstructProp::Value(v) => Ok(v),
        }
    }
}

/// Construct driver trait
pub trait Construct: Sized {
    /// Props
    type Props: Default + Clone;

    /// Construct an item.
    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError>;
}

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

// TODO: This blanket impl is mentioned in the scene discussion, but is ambigous with custom impl Construct, and tuple impls.
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

// Workaround, implement for some Bevy components
macro_rules! impl_construct_passthrough {
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
impl_construct_passthrough!(Transform);
impl_construct_passthrough!(Node);
impl_construct_passthrough!(BackgroundColor);
impl_construct_passthrough!(BorderColor);
impl_construct_passthrough!(BorderRadius);
impl_construct_passthrough!(Text);
impl_construct_passthrough!(TextFont);
impl_construct_passthrough!(TextColor);

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

/// An entity and a mutable world
#[derive(Debug)]
pub struct ConstructContext<'a> {
    /// Entity to use for construction
    pub id: Entity,
    /// World
    pub world: &'a mut World,
}

impl<'a> ConstructContext<'a> {
    /// Construct helper function
    pub fn construct<T: Construct>(
        &mut self,
        props: impl Into<T::Props>,
    ) -> Result<T, ConstructError> {
        T::construct(self, props.into())
    }
}

struct ConstructCommand<T: Construct>(T::Props);

impl<T: Construct + Bundle> EntityCommand for ConstructCommand<T>
where
    <T as Construct>::Props: Send,
{
    fn apply(self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
        let c = T::construct(&mut context, self.0).expect("component");
        world.entity_mut(id).insert(c);
    }
}

/// Construct extension
pub trait ConstructExt {
    /// Construct a type using the given properties and insert it onto the entity.
    fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityCommands
    where
        <T as Construct>::Props: Send;
}

impl<'w> ConstructExt for Commands<'w, '_> {
    // type Out = EntityCommands;
    fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityCommands
    where
        <T as Construct>::Props: Send,
    {
        let mut s = self.spawn_empty();
        s.queue(ConstructCommand::<T>(props.into()));
        s
    }
}

impl<'w> ConstructExt for ChildBuilder<'w> {
    // type Out = EntityCommands;
    fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityCommands
    where
        <T as Construct>::Props: Send,
    {
        let mut s = self.spawn_empty();
        s.queue(ConstructCommand::<T>(props.into()));
        s
    }
}

impl<'w> ConstructExt for EntityCommands<'w> {
    // type Out = EntityCommands;
    fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityCommands
    where
        <T as Construct>::Props: Send,
    {
        self.queue(ConstructCommand::<T>(props.into()));
        self.reborrow()
    }
}
