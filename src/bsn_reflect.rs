use bevy::{
    app::App,
    reflect::{FromType, Reflect},
};

use crate::parse::{syn::Expr, FromBsn, FromBsnError};

/// A struct used to operate on reflected [`FromBsn`] trait of a type.
///
/// A [`ReflectFromBsn`] for type `T` can be obtained via [`FromType::from_type`].
#[derive(Clone)]
pub struct ReflectFromBsn {
    /// Function to convert a BSN expression to a [`Reflect`] value.
    pub from_bsn: for<'a> fn(Expr) -> Result<Box<dyn Reflect>, FromBsnError>,
}

impl ReflectFromBsn {
    /// Converts a BSN expression to a [`Reflect`] value.
    pub fn from_bsn(&self, value: Expr) -> Result<Box<dyn Reflect>, FromBsnError> {
        (self.from_bsn)(value)
    }
}

impl<F: FromBsn + Reflect> FromType<F> for ReflectFromBsn {
    fn from_type() -> Self {
        Self {
            from_bsn: |value| Ok(Box::new(F::from_bsn(value)?)),
        }
    }
}

pub(crate) fn register_reflect_from_bsn(app: &mut App) {
    app.register_type_data::<(), ReflectFromBsn>();
    app.register_type_data::<u8, ReflectFromBsn>();
    app.register_type_data::<u16, ReflectFromBsn>();
    app.register_type_data::<u32, ReflectFromBsn>();
    app.register_type_data::<u64, ReflectFromBsn>();
    app.register_type_data::<u128, ReflectFromBsn>();
    app.register_type_data::<usize, ReflectFromBsn>();
    app.register_type_data::<i8, ReflectFromBsn>();
    app.register_type_data::<i16, ReflectFromBsn>();
    app.register_type_data::<i32, ReflectFromBsn>();
    app.register_type_data::<i64, ReflectFromBsn>();
    app.register_type_data::<i128, ReflectFromBsn>();
    app.register_type_data::<f32, ReflectFromBsn>();
    app.register_type_data::<f64, ReflectFromBsn>();
    app.register_type_data::<String, ReflectFromBsn>();
    app.register_type_data::<bool, ReflectFromBsn>();
}
