mod class;
mod constant;
mod extern_;
mod from_zval;
mod function;
mod impl_;
mod into_zval;
mod method;
mod module;
mod startup_function;

use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use constant::Constant;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse_macro_input, AttributeArgs, DeriveInput, ItemConst, ItemFn, ItemForeignMod, ItemImpl,
    ItemStruct,
};

extern crate proc_macro;

#[derive(Default, Debug)]
struct State {
    functions: Vec<function::Function>,
    classes: HashMap<String, class::Class>,
    constants: Vec<Constant>,
    startup_function: Option<String>,
    built_module: bool,
}

lazy_static::lazy_static! {
    pub(crate) static ref STATE: StateMutex = StateMutex::new();
}

struct StateMutex(Mutex<State>);

impl StateMutex {
    pub fn new() -> Self {
        Self(Mutex::new(Default::default()))
    }

    pub fn lock(&self) -> MutexGuard<State> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[proc_macro_attribute]
pub fn php_class(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    match class::parser(args, input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    match function::parser(args, input) {
        Ok((parsed, _)) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_module(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    match module::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_startup(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    match startup_function::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemImpl);

    match impl_::parser(args, input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_const(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemConst);

    match constant::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemForeignMod);

    match extern_::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

// TODO(david): potentially add option to ignore field in exchange for `Default::default()`?
#[proc_macro_derive(FromZval)]
pub fn from_zval_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match from_zval::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(IntoZval)]
pub fn into_zval_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match into_zval::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}
