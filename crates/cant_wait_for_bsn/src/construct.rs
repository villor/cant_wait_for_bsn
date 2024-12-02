//! Based on implementation from [bevy_asky](https://github.com/shanecelis/bevy_asky/blob/main/src/construct.rs).
use bevy::utils::all_tuples;
use bevy::{ecs::system::EntityCommands, prelude::*};
use std::borrow::Cow;
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
    /// Consumes the [`ConstructProp`] and returns the (constructed) value.
    ///
    /// Added for convenience.
    pub fn construct(self, context: &mut ConstructContext) -> Result<T, ConstructError> {
        match self {
            ConstructProp::Prop(p) => Construct::construct(context, p),
            ConstructProp::Value(v) => Ok(v),
        }
    }
}

/// Construct driver trait
pub trait Construct: Sized {
    /// Properties must be Clone.
    ///
    /// NOTE: Cart's proposal states they must also be Default,
    /// but I had trouble making that work.
    type Props: Default + Clone;

    /// Construct an item.
    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError>;
}

/// Add a silent partner.
// #[derive(Bundle)]
// pub struct Add<A: Sync + Send + 'static + Bundle, B: Sync + Send + 'static + Bundle>(pub A, pub B);

// unsafe impl<A: Submitter + Sync + Send + 'static + Bundle, B: Sync + Send + 'static + Bundle>
//     Submitter for Add<A, B>
// {
//     /// Output of submitter.
//     type Out = A::Out;
// }

// impl<A, B> Construct for Add<A, B>
// where
//     A: Construct + Sync + Send + 'static + Bundle,
//     B: Construct<Props = ()> + Sync + Send + 'static + Bundle,
// {
//     type Props = A::Props;
//     fn construct(
//         context: &mut ConstructContext,
//         props: Self::Props,
//     ) -> Result<Self, ConstructError> {
//         let a = A::construct(context, props)?;
//         let b = B::construct(context, ())?;
//         Ok(Add(a, b))
//     }
// }

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

// I couldn't have this an the tuple construct.
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
    ($(($T:ident, $t:ident)),*) => {
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

all_tuples!(impl_construct_for_tuple, 0, 12, T, t);

// impl Construct for () {
//     type Props = ();
//     #[inline]
//     fn construct(
//         _context: &mut ConstructContext,
//         props: Self::Props,
//     ) -> Result<Self, ConstructError> {
//         Ok(props)
//     }
// }

// impl<A: Construct, B: Construct> Construct for (A, B)
// // where
// //     A: Construct,
// //     B: Construct,
// {
//     type Props = (A::Props, B::Props);
//     fn construct(
//         context: &mut ConstructContext,
//         props: Self::Props,
//     ) -> Result<Self, ConstructError> {
//         let (A, B) = props;
//         let A = context.construct(A)?;
//         let B = context.construct(B)?;
//         Ok((A, B))
//     }
// }

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

/// Construct extension
///
/// The main touch point for the user.
pub trait ConstructExt {
    /// Construct a type using the given properties.
    fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityCommands
    where
        <T as Construct>::Props: Send;
}

/// Construct children extension
pub trait ConstructChildrenExt: ConstructExt {
    /// Construct a series of children using the given properties.
    fn construct_children<T: Construct + Bundle>(
        &mut self,
        props: impl IntoIterator<Item = impl Into<T::Props>>,
    ) -> EntityCommands
    where
        <T as Construct>::Props: Send;
}

struct ConstructCommand<T: Construct>(T::Props);

impl<T: Construct + Bundle> bevy::ecs::system::EntityCommand for ConstructCommand<T>
where
    <T as Construct>::Props: Send,
{
    fn apply(self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
        let c = T::construct(&mut context, self.0).expect("component");
        world.entity_mut(id).insert(c);
    }
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

// impl<'w> ConstructExt for EntityWorldMut<'w> {
//     // type Out = EntityCommands;
//     fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityWorldMut<'w>
//     where
//         <T as Construct>::Props: Send,
//     {
//         let ctx = ConstructContext {
//             id: self.id(),
//             world: unsafe { self.world_mut() }, // SAFETY: TODO
//         };

//         // Construct::construct()
//         // self.queue(ConstructCommand::<T>(props.into()));
//         // self.reborrow()
//     }
// }

impl<'w> ConstructChildrenExt for EntityCommands<'w> {
    fn construct_children<T: Construct + Bundle>(
        &mut self,
        props: impl IntoIterator<Item = impl Into<T::Props>>,
    ) -> EntityCommands
    where
        <T as Construct>::Props: Send,
    {
        self.with_children(|parent| {
            for prop in props.into_iter() {
                parent.construct::<T>(prop);
            }
        });
        self.reborrow()
    }
}
