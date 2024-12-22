//! Based on FromBsn from Cart's first proposal: https://github.com/cart/bevy/commit/d5b84bd577c8f6f07eedaf7d394823644c116aa4#diff-d2a66394968486178ae32844c15e7e6df454fd5ce0cd6d581b7a521c38436d26
//!
//! Currently parses from syn::Expr. But could be based on a custom AST in the future.
//use bevy_math::{Quat, Vec2, Vec3};
use syn::{Expr, ExprLit, Lit};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FromBsnError {
    #[error("Type did not match expected type")]
    MismatchedType,
    #[error("Encountered unexpected field {0}")]
    UnexpectedField(String),
    #[error(transparent)]
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

/// Allows a type to be parsed from a BSN expression.
pub trait FromBsn: Sized {
    fn from_bsn(value: Expr) -> Result<Self, FromBsnError>;
}

impl FromBsn for () {
    fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
        if let Expr::Tuple(tuple) = value {
            if tuple.elems.is_empty() {
                Ok(())
            } else {
                Err(FromBsnError::MismatchedType)
            }
        } else {
            Err(FromBsnError::MismatchedType)
        }
    }
}

macro_rules! impl_with_parse {
    ($ty: ident) => {
        impl FromBsn for $ty {
            fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
                if let Expr::Lit(ExprLit { lit, .. }) = value {
                    match lit {
                        Lit::Int(value) => Ok(value
                            .base10_parse()
                            .map_err(|e| FromBsnError::Custom(Box::new(e)))?),
                        Lit::Float(value) => Ok(value
                            .base10_parse()
                            .map_err(|e| FromBsnError::Custom(Box::new(e)))?),
                        _ => Err(FromBsnError::MismatchedType),
                    }
                } else {
                    Err(FromBsnError::MismatchedType)
                }
            }
        }
    };
}

impl_with_parse!(u8);
impl_with_parse!(u16);
impl_with_parse!(u32);
impl_with_parse!(u64);
impl_with_parse!(u128);
impl_with_parse!(usize);
impl_with_parse!(i8);
impl_with_parse!(i16);
impl_with_parse!(i32);
impl_with_parse!(i64);
impl_with_parse!(i128);
impl_with_parse!(f32);
impl_with_parse!(f64);

impl FromBsn for String {
    fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) = value
        {
            Ok(value.value())
        } else {
            Err(FromBsnError::MismatchedType)
        }
    }
}

impl FromBsn for bool {
    fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) = value
        {
            Ok(value.value)
        } else {
            Err(FromBsnError::MismatchedType)
        }
    }
}

// TODO:
// impl FromBsn for Vec2 {
//     fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
//         if let Expr::Struct(bsn_struct) = value {
//             let mut value = Self::default();
//             match bsn_struct {
//                 crate::BsnStruct::Tuple(_) => {}
//                 crate::BsnStruct::NamedFields(fields) => {
//                     for field in fields {
//                         match field.name {
//                             "x" => {
//                                 value.x = f32::from_bsn(field.value)?;
//                             }
//                             "y" => {
//                                 value.y = f32::from_bsn(field.value)?;
//                             }
//                             _ => return Err(FromBsnError::UnexpectedField(field.name.to_string())),
//                         }
//                     }
//                 }
//             }
//             Ok(value)
//         } else {
//             Err(FromBsnError::MismatchedType)
//         }
//     }
// }

// impl FromBsn for Vec3 {
//     fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
//         if let Expr::Struct(bsn_struct) = value {
//             let mut value = Self::default();
//             match bsn_struct {
//                 crate::BsnStruct::Tuple(_) => {}
//                 crate::BsnStruct::NamedFields(fields) => {
//                     for field in fields {
//                         match field.name {
//                             "x" => {
//                                 value.x = f32::from_bsn(field.value)?;
//                             }
//                             "y" => {
//                                 value.y = f32::from_bsn(field.value)?;
//                             }
//                             "z" => {
//                                 value.z = f32::from_bsn(field.value)?;
//                             }
//                             _ => return Err(FromBsnError::UnexpectedField(field.name.to_string())),
//                         }
//                     }
//                 }
//             }
//             Ok(value)
//         } else {
//             Err(FromBsnError::MismatchedType)
//         }
//     }
// }

// impl FromBsn for Quat {
//     fn from_bsn(value: Expr) -> Result<Self, FromBsnError> {
//         if let Expr::Struct(bsn_struct) = value {
//             let mut value = Self::default();
//             match bsn_struct {
//                 crate::BsnStruct::Tuple(_) => {}
//                 crate::BsnStruct::NamedFields(fields) => {
//                     for field in fields {
//                         match field.name {
//                             "w" => {
//                                 value.w = f32::from_bsn(field.value)?;
//                             }
//                             "x" => {
//                                 value.x = f32::from_bsn(field.value)?;
//                             }
//                             "y" => {
//                                 value.y = f32::from_bsn(field.value)?;
//                             }
//                             "z" => {
//                                 value.z = f32::from_bsn(field.value)?;
//                             }
//                             _ => return Err(FromBsnError::UnexpectedField(field.name.to_string())),
//                         }
//                     }
//                 }
//             }
//             Ok(value)
//         } else {
//             Err(FromBsnError::MismatchedType)
//         }
//     }
// }
