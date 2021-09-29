use anyhow::{bail, Context, Result};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    token::Where, DataEnum, DataStruct, DeriveInput, GenericParam, Ident, ImplGenerics,
    TypeGenerics, WhereClause,
};

pub fn parser(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        generics, ident, ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: Where {
            span: Span::call_site(),
        },
        predicates: Default::default(),
    });

    for generic in generics.params.iter() {
        if let GenericParam::Type(ty) = generic {
            let ident = &ty.ident;
            where_clause.predicates.push(
                syn::parse2(quote! {
                    #ident: ::ext_php_rs::php::types::zval::IntoZval
                })
                .expect("couldn't parse where predicate"),
            );
        }
    }

    match input.data {
        syn::Data::Struct(data) => {
            parse_struct(data, ident, impl_generics, ty_generics, where_clause)
        }
        syn::Data::Enum(data) => parse_enum(data, ident, impl_generics, ty_generics, where_clause),
        _ => bail!("Only structs and enums are supported by the `#[derive(IntoZval)]` macro."),
    }
}

fn parse_struct(
    data: DataStruct,
    ident: Ident,
    impl_generics: ImplGenerics,
    ty_generics: TypeGenerics,
    where_clause: WhereClause,
) -> Result<TokenStream> {
    let fields = data
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().with_context(|| {
                "Fields require names when using the `#[derive(IntoZval)]` macro on a struct."
            })?;
            let field_name = ident.to_string();

            Ok(quote! {
                obj.set_property(#field_name, self.#ident)?;
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl #impl_generics ::ext_php_rs::php::types::zval::IntoZval for #ident #ty_generics #where_clause {
            const TYPE: ::ext_php_rs::php::enums::DataType = ::ext_php_rs::php::enums::DataType::Object(::std::option::Option::None);

            fn set_zval(
                self,
                zv: &mut ::ext_php_rs::php::types::zval::Zval,
                persistent: bool,
            ) -> ::ext_php_rs::errors::Result<()> {
                use ::ext_php_rs::php::types::zval::IntoZval;

                let mut obj = ::ext_php_rs::php::types::object::OwnedZendObject::new_stdclass();
                #(#fields)*
                obj.set_zval(zv, persistent)
            }
        }
    })
}

fn parse_enum(
    data: DataEnum,
    ident: Ident,
    impl_generics: ImplGenerics,
    ty_generics: TypeGenerics,
    where_clause: WhereClause,
) -> Result<TokenStream> {
    let variants = data.variants.iter().filter_map(|variant| {
        // can have default fields - in this case, return `null`.
        if variant.fields.len() != 1 {
            return None;
        }

        let variant_ident = &variant.ident;
        Some(quote! {
            #ident::#variant_ident(val) => val.set_zval(zv, persistent)
        })
    });

    Ok(quote! {
        impl #impl_generics ::ext_php_rs::php::types::zval::IntoZval for #ident #ty_generics #where_clause {
            const TYPE: ::ext_php_rs::php::enums::DataType = ::ext_php_rs::php::enums::DataType::Mixed;

            fn set_zval(
                self,
                zv: &mut ::ext_php_rs::php::types::zval::Zval,
                persistent: bool,
            ) -> ::ext_php_rs::errors::Result<()> {
                use ::ext_php_rs::php::types::zval::IntoZval;

                match self {
                    #(#variants,)*
                    _ => {
                        zv.set_null();
                        Ok(())
                    }
                }
            }
        }
    })
}
