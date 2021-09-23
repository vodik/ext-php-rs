use anyhow::{bail, Context, Result};
use darling::ToTokens;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::token::Where;
use syn::{
    DataEnum, DataStruct, DeriveInput, GenericParam, Generics, Lifetime, LifetimeDef, TypeGenerics,
    Variant, WhereClause,
};

pub fn parser(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        generics, ident, ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // FIXME(david): work around since mutating `generics` will add the lifetime to the struct generics as well,
    // leading to an error as we would have `impl<'a> FromZval<'a> for Struct<'a>` when `Struct` has no lifetime.
    let impl_generics = {
        let tokens = impl_generics.to_token_stream();
        let mut parsed: Generics = syn::parse2(tokens).expect("couldn't reparse generics");
        parsed
            .params
            .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                "'_zval",
                Span::call_site(),
            ))));
        parsed
    };

    let mut where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: Where {
            span: Span::call_site(),
        },
        predicates: Default::default(),
    });

    for generic in generics.params.iter() {
        match generic {
            GenericParam::Type(ty) => {
                let ident = &ty.ident;
                where_clause.predicates.push(
                    syn::parse2(quote! {
                        #ident: ::ext_php_rs::php::types::zval::FromZval<'_zval>
                    })
                    .expect("couldn't parse where predicate"),
                )
            }
            GenericParam::Lifetime(lt) => where_clause.predicates.push(
                syn::parse2(quote! {
                    '_zval: #lt
                })
                .expect("couldn't parse where predicate"),
            ),
            _ => continue,
        }
    }

    match input.data {
        syn::Data::Struct(data) => {
            parse_struct(data, ident, impl_generics, ty_generics, where_clause)
        }
        syn::Data::Enum(data) => parse_enum(data, ident, impl_generics, ty_generics, where_clause),
        _ => bail!("Only structs and enums are supported by the `#[derive(FromZval)]` macro."),
    }
}

fn parse_struct(
    data: DataStruct,
    ident: Ident,
    impl_generics: Generics,
    ty_generics: TypeGenerics,
    where_clause: WhereClause,
) -> Result<TokenStream> {
    let fields = data
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().with_context(|| {
                "Fields require names when using the `#[derive(FromZval)]` macro on a struct."
            })?;
            let field_name = ident.to_string();

            Ok(quote! {
                #ident: obj.get_property(#field_name).ok()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl #impl_generics ::ext_php_rs::php::types::zval::FromZval<'_zval> for #ident #ty_generics #where_clause {
            const TYPE: ::ext_php_rs::php::enums::DataType = ::ext_php_rs::php::enums::DataType::Object(::std::option::Option::None);

            fn from_zval(zval: &'_zval ::ext_php_rs::php::types::zval::Zval) -> ::std::option::Option<Self> {
                let obj = zval.object()?;

                Some(Self {
                    #(#fields)*
                })
            }
        }
    })
}

fn parse_enum(
    data: DataEnum,
    ident: Ident,
    impl_generics: Generics,
    ty_generics: TypeGenerics,
    where_clause: WhereClause,
) -> Result<TokenStream> {
    let mut default = None;
    let variants = data.variants.iter().map(|variant| {
        let Variant {
            ident,
            fields,
            ..
        } = variant;

        match fields {
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    bail!("Enum variant must only have one field when using `#[derive(FromZval)]`.");
                }

                let ty = &fields.unnamed.first().unwrap().ty;

                Ok(Some(quote! {
                    if let Some(value) = <#ty>::from_zval(zval) {
                        return Some(Self::#ident(value));
                    }
                }))
            },
            syn::Fields::Unit => {
                if default.is_some() {
                    bail!("Only one enum unit type is valid as a default when using `#[derive(FromZval)]`.");
                }

                default.replace(quote! {
                    Some(Self::#ident)
                });
                Ok(None)
            }
            _ => bail!("Enum variants must be unnamed and have only one field inside the variant when using `#[derive(FromZval)]`.")
        }
    }).collect::<Result<Vec<_>>>()?;

    let default = default.unwrap_or_else(|| quote! { None });

    Ok(quote! {
        impl #impl_generics ::ext_php_rs::php::types::zval::FromZval<'_zval> for #ident #ty_generics #where_clause {
            const TYPE: ::ext_php_rs::php::enums::DataType = ::ext_php_rs::php::enums::DataType::Mixed;

            fn from_zval(zval: &'_zval ::ext_php_rs::php::types::zval::Zval) -> ::std::option::Option<Self> {
                #(#variants)*
                #default
            }
        }
    })
}
