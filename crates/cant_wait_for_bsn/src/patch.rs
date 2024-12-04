//! Based on implementation from [bevy_asky](https://github.com/shanecelis/bevy_asky/blob/main/src/construct.rs).
use bevy::{prelude::*, utils::all_tuples};
use core::marker::PhantomData;

use crate::construct::{Construct, ConstructContext, ConstructError};

/// Modifies properties
pub trait Patch: Send + Sync + 'static {
    /// Of what type
    type Construct: Construct + Bundle;
    /// Modify properties
    fn patch(&mut self, props: &mut <Self::Construct as Construct>::Props);
}

// impl Patch for () {
//     type Construct = ();
//     fn patch(&mut self, _props: &mut <Self::Construct as Construct>::Props) {}
// }

// impl<P: Patch> Patch for (P,) {
//     type Construct = P::Construct;

/// Used to aid tuple implementations for [`Patch`].
#[derive(Bundle, Deref, DerefMut)]
pub struct PatchTupleConstruct<C: Bundle + Send + Sync>(C);

// Tuple impls
macro_rules! impl_patch_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: Construct + Bundle),*> Construct for PatchTupleConstruct<($($T,)*)> {
            type Props = ($(<$T as Construct>::Props,)*);

            fn construct(
                _context: &mut ConstructContext,
                props: Self::Props,
            ) -> Result<Self, ConstructError> {
                let ($($t,)*) = props;
                $(let $t = $T::construct(_context, $t)?;)*
                Ok(Self(($($t,)*)))
            }
        }

        $(#[$meta])*
        impl<$($T: Patch),*> Patch for ($($T,)*) {
            type Construct = PatchTupleConstruct<($($T::Construct,)*)>;

            #[allow(non_snake_case)]
            fn patch(&mut self, props: &mut <Self::Construct as Construct>::Props) {
                let ($($T,)*) = self;
                let ($($t,)*) = props;
                $($T.patch($t);)*
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_patch_for_tuple,
    0,
    12,
    T,
    t
);

/// Generic patch based on closure
pub struct ConstructPatch<C: Construct, F> {
    func: F,
    _marker: PhantomData<C>,
}

impl<C: Construct + Bundle, F: FnMut(&mut C::Props) + Sync + Send + 'static> Patch
    for ConstructPatch<C, F>
{
    type Construct = C;
    fn patch(&mut self, props: &mut <Self::Construct as Construct>::Props) {
        (self.func)(props);
    }
}

/// Extension trait for adding a [`ConstructPatchExt::patch`] utility to any types implementing [`Construct`].
pub trait ConstructPatchExt {
    /// Construct
    type C: Construct;

    /// Returns a [`ConstructPatch`] wrapping the provided closure.
    fn patch<F: FnMut(&mut <<Self as ConstructPatchExt>::C as Construct>::Props)>(
        func: F,
    ) -> ConstructPatch<Self::C, F> {
        ConstructPatch {
            func,
            _marker: PhantomData,
        }
    }
}

impl<C: Construct> ConstructPatchExt for C {
    type C = C;
}

/// Cloned patch. Wraps a value that will be cloned on patch to overwrite the props.
// pub struct ClonedPatch<T: Construct + Bundle>(T::Props);

// impl<T: Construct + Bundle> ClonedPatch<T>
// where
//     T: Construct<Props = T>,
// {
//     /// Construct a [`ClonedPatch`]. Only works for types where the construct and props have the same type.
//     pub fn new(props: T) -> Self {
//         Self(props)
//     }
// }

// impl<T: Construct + Bundle> Patch for ClonedPatch<T>
// where
//     <T as Construct>::Props: Sync + Send,
// {
//     type Construct = T;
//     fn patch(&mut self, props: &mut <Self::Construct as Construct>::Props) {
//         *props = self.0.clone();
//     }
// }

/// Extension trait implementing patch utilities for [`ConstructContext`].
pub trait ConstructContextPatchExt {
    /// Construct from patch
    fn construct_from_patch<P: Patch>(
        &mut self,
        patch: &mut P,
    ) -> Result<P::Construct, ConstructError>
    where
        <<P as Patch>::Construct as Construct>::Props: Default;
}

impl<'a> ConstructContextPatchExt for ConstructContext<'a> {
    fn construct_from_patch<P: Patch>(
        &mut self,
        patch: &mut P,
    ) -> Result<P::Construct, ConstructError>
    where
        <<P as Patch>::Construct as Construct>::Props: Default,
    {
        let mut props = <<P as Patch>::Construct as Construct>::Props::default();
        patch.patch(&mut props);
        self.construct(props)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Component)]
    struct Player {
        _name: String,
    }

    #[derive(Default, Clone)]
    struct PlayerProps {
        name: String,
    }

    impl Construct for Player {
        type Props = PlayerProps;
        fn construct(
            _context: &mut ConstructContext,
            props: Self::Props,
        ) -> Result<Self, ConstructError> {
            Ok(Player { _name: props.name })
        }
    }

    #[test]
    fn test_patch_name() {
        let mut player = PlayerProps {
            name: "shane".into(),
        };
        assert_eq!(player.name, "shane");

        let mut patch = Player::patch(|props| {
            props.name = "fred".to_string();
        });
        patch.patch(&mut player);
        assert_eq!(player.name, "fred");
    }
}
