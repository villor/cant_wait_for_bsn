use core::any::TypeId;

use bevy::{
    app::App,
    reflect::{FromType, PartialReflect, Reflect, Reflectable, TypePath},
};

use crate::{Construct, ConstructContext, ConstructError};

/// A struct used to operate on reflected [`Construct`] trait of a type.
///
/// A [`ReflectConstruct`] for type `T` can be obtained via [`FromType::from_type`].
#[derive(Clone)]
pub struct ReflectConstruct {
    construct: fn(
        &mut ConstructContext,
        Box<dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, ConstructError>,
    default_props: fn() -> Box<dyn Reflect>,
    /// The type id of the props type.
    pub props_type_id: TypeId,
}

impl ReflectConstruct {
    /// Constructs a value by calling `T::construct` with the given dynamic props.
    pub fn construct(
        &self,
        context: &mut ConstructContext,
        props: Box<dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, ConstructError> {
        (self.construct)(context, props)
    }

    /// Returns the default props for this type.
    pub fn default_props(&self) -> Box<dyn Reflect> {
        (self.default_props)()
    }
}

impl<T: Construct + Reflectable> FromType<T> for ReflectConstruct
where
    <T as Construct>::Props: Reflect + TypePath,
{
    fn from_type() -> Self {
        ReflectConstruct {
            construct: |context, props| {
                let Ok(props) = props.take::<T::Props>() else {
                    return Err(ConstructError::InvalidProps {
                        message: format!("failed to downcast props to {}", T::Props::type_path())
                            .into(),
                    });
                };

                let constructed = T::construct(context, props)?;
                Ok(Box::new(constructed))
            },
            default_props: || Box::new(T::Props::default()),
            props_type_id: TypeId::of::<T::Props>(),
        }
    }
}

pub(crate) fn register_reflect_construct(app: &mut App) {
    use bevy::prelude::*;

    // Transform and visibility
    app.register_type_data::<Transform, ReflectConstruct>();
    app.register_type_data::<GlobalTransform, ReflectConstruct>();
    app.register_type_data::<Visibility, ReflectConstruct>();
    app.register_type_data::<InheritedVisibility, ReflectConstruct>();

    // UI components
    app.register_type_data::<Node, ReflectConstruct>();
    app.register_type_data::<BorderColor, ReflectConstruct>();
    app.register_type_data::<BorderRadius, ReflectConstruct>();
    app.register_type_data::<BackgroundColor, ReflectConstruct>();

    // UI widgets
    app.register_type_data::<Button, ReflectConstruct>();
    app.register_type_data::<Label, ReflectConstruct>();
    app.register_type_data::<Text, ReflectConstruct>();

    // Text
    app.register_type_data::<Text2d, ReflectConstruct>();
    app.register_type_data::<TextFont, ReflectConstruct>();

    // TODO: Add all of em!
}
