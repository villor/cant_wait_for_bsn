use bevy_macro_utils::{
    fq_std::{FQClone, FQDefault, FQResult},
    BevyManifest,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse2, parse_quote, Data, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Path,
    Visibility,
};

pub fn derive_construct(item: TokenStream) -> TokenStream {
    match parse2::<DeriveInput>(item) {
        Ok(s) => impl_construct(s),
        Err(e) => e.to_compile_error(),
    }
}

fn impl_construct(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        vis,
        ident,
        generics,
        data,
        ..
    } = input;

    match data {
        Data::Struct(ds) => impl_for_struct(vis, ident, generics, ds),
        Data::Enum(_) => todo!("enums are not supported"),
        Data::Union(_) => todo!("unions are not supported"),
    }
}

fn impl_for_struct(
    vis: Visibility,
    ident: Ident,
    _generics: Generics, // TODO
    data_struct: DataStruct,
) -> TokenStream {
    let props_ident = Ident::new(format!("{}{}", ident, "Props").as_str(), ident.span());

    let bevy_manifest = BevyManifest::default();
    let bevy_reflect = bevy_manifest.get_path("bevy_reflect");

    let cant_wait_for_bsn = Path::from(Ident::new("cant_wait_for_bsn", Span::call_site()));

    let fields: Vec<(&Field, bool)> = data_struct
        .fields
        .iter()
        .map(|field| {
            let is_construct = field.attrs.iter().any(|attr| {
                attr.meta
                    .path()
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .unwrap_or_default()
                    == "construct"
            });
            (field, is_construct)
        })
        .collect();

    let struct_fields = fields.iter().map(|(field, is_construct)| {
        let mut field = Field::clone(field);
        if *is_construct {
            let ty = field.ty;
            field.ty = parse_quote! { #cant_wait_for_bsn::ConstructProp<#ty> };
        }
        field.attrs.clear();
        field
    });

    let default_fields = fields.iter().map(|(field, is_construct)| {
        let Field {
            ident, colon_token, ..
        } = field;
        if *is_construct {
            quote! { #ident #colon_token #cant_wait_for_bsn::ConstructProp::Prop(#FQDefault::default()) }
        } else {
            quote! { #ident #colon_token #FQDefault::default() }
        }
    });

    let construct_fields = fields.iter().map(|(field, is_construct)| {
        let Field {
            ident, colon_token, ..
        } = field;
        if *is_construct {
            quote! { #ident #colon_token match props.#ident {
                #cant_wait_for_bsn::ConstructProp::Prop(p) => {
                    #cant_wait_for_bsn::Construct::construct(p, context)?
                },
                #cant_wait_for_bsn::ConstructProp::Value(v) => v,
            } }
        } else {
            quote! { #ident #colon_token props.#ident }
        }
    });

    match data_struct.fields {
        Fields::Unit => quote! {
            #[allow(missing_docs)]
            #[derive(#FQClone, #bevy_reflect::Reflect, #FQDefault)]
            #vis struct #props_ident;

            impl #cant_wait_for_bsn::Construct for #ident {
                type Props = #props_ident;
                fn construct(context: &mut #cant_wait_for_bsn::ConstructContext, props: Self::Props) -> #FQResult<Self, #cant_wait_for_bsn::ConstructError> {
                    Ok(Self)
                }
            }
        },
        Fields::Named(_) => quote! {
            #[allow(missing_docs)]
            #[derive(#FQClone, #bevy_reflect::Reflect)]
            #vis struct #props_ident {
                #(#struct_fields),*
            }

            impl #FQDefault for #props_ident {
                fn default() -> Self {
                    Self { #(#default_fields),* }
                }
            }

            impl #cant_wait_for_bsn::Construct for #ident {
                type Props = #props_ident;
                fn construct(context: &mut #cant_wait_for_bsn::ConstructContext, props: Self::Props) -> #FQResult<Self, #cant_wait_for_bsn::ConstructError> {
                    Ok(Self { #(#construct_fields),* })
                }
            }
        },
        Fields::Unnamed(_) => quote! {
            #[allow(missing_docs)]
            #[derive(#FQClone, #bevy_reflect::Reflect)]
            #vis struct #props_ident ( #(#struct_fields),* );

            impl #FQDefault for #props_ident {
                fn default() -> Self {
                    Self( #(#default_fields),* )
                }
            }

            impl #cant_wait_for_bsn::Construct for #ident {
                type Props = #props_ident;
                fn construct(context: &mut #cant_wait_for_bsn::ConstructContext, props: Self::Props) -> #FQResult<Self, #cant_wait_for_bsn::ConstructError> {
                    Ok(Self ( #(#construct_fields),* ))
                }
            }
        },
    }
}
