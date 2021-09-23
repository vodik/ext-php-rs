use anyhow::{bail, Context, Result};
use darling::ToTokens;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::token::Where;
use syn::{DeriveInput, GenericParam, Generics, Lifetime, LifetimeDef, WhereClause};

pub fn parser(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_) => parse_struct(input),
        syn::Data::Enum(_) => parse_enum(input),
        _ => bail!("Only structs and enums are supported by the `#[derive(FromZval)]` macro."),
    }
}

fn parse_struct(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        data,
        generics,
        ident,
        ..
    } = input;
    let data = match data {
        syn::Data::Struct(data) => data,
        _ => unreachable!(),
    };

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

    let mut where_clause = where_clause
        .map(|w| w.clone())
        .unwrap_or_else(|| WhereClause {
            where_token: Where {
                span: Span::call_site(),
            },
            predicates: Default::default(),
        });

    for generic in impl_generics.params.iter() {
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
            _ => continue,
        }
    }

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

fn parse_enum(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        data,
        attrs,
        vis,
        generics,
        ident,
    } = input;
    Ok(quote! {
        impl ::ext_php_rs::php::types::zval::FromZval<'_> for #ident {
            const TYPE: ::ext_php_rs::php::enums::DataType = ::ext_php_rs::php::enums::DataType::Object(::std::option::Option::None);

            fn from_zval(zval: &::ext_php_rs::php::types::zval::Zval) -> ::std::option::Option<Self> {

            }
        }
    })
}
