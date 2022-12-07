extern crate core;

use convert_case::{Case, Casing};
use darling::util::{Flag, Override};
use darling::FromVariant;
use proc_macro::{self, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Path, Variant};

/// Get the value of the path segment
fn to_value(variant: &Variant, opts: &Opts) -> String {
    if opts.index.is_present() {
        return "".to_string();
    }

    opts.rename
        .clone()
        .unwrap_or_else(|| variant.ident.to_string().to_lowercase())
}

#[derive(FromVariant, Default)]
#[darling(default, attributes(target))]
struct Opts {
    index: Flag,
    rename: Option<String>,
    default: Option<Override<String>>,
}

#[proc_macro_derive(Target, attributes(target))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let data = match data {
        Data::Enum(e) => e,
        _ => panic!("Derive must be used on enum only"),
    };

    let render_path = data.variants.iter().map(|v| {
        let name = &v.ident;

        match &v.fields {
            Fields::Unit => {
                quote! {
                    Self::#name => {
                        self.render_self_into(path);
                    }
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    Self::#name(nested) => {
                        self.render_self_into(path);
                        nested.render_path_into(path);
                    }
                }
            }
            Fields::Named(_) => {
                quote! {
                    Self::#name{..} => {
                        self.render_self_into(path);
                    }
                }
            }
        }
    });

    let render_self = data.variants.iter().map(|v| {
        let name = &v.ident;

        let opts = Opts::from_variant(v).expect("Unable to parse options");
        let value = to_value(&v, &opts);

        match &v.fields {
            // plain route
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    Self::#name => path.push(#value.into())
                }
            }
            // nested route
            Fields::Unnamed(_) => {
                quote_spanned! { v.span() =>
                    Self::#name(_) => path.push(#value.into())
                }
            }
            // variables
            Fields::Named(variables) => {
                let vars = variables
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().expect("Tuples are not supported"));
                let values = variables.named.iter().map(|f| {
                    let ident = f.ident.as_ref().expect("Tuples are not supported");
                    quote! {
                        #ident.into()
                    }
                });

                quote_spanned! { v.span() =>
                    Self::#name { #(#vars),* } => {
                        path.push(#value.into());
                        path.extend([#(#values),*]);
                    }
                }
            }
        }
    });

    let parse_path = data.variants.iter().map(|v| {
        let name = &v.ident;

        let opts = Opts::from_variant(v).expect("Unable to parse options");
        let value = to_value(&v, &opts);

        match &v.fields {
            // plain route
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    [#value] => Some(Self::#name)
                }
            }
            // nested route
            Fields::Unnamed(nested) => {
                if nested.unnamed.len() > 1 {
                    panic!(
                        "Only one unnamed variable is supported, which is the nested target type"
                    );
                }

                match nested.unnamed.first() {
                    Some(f) => {
                        let t = &f.ty;
                        let default = match &opts.default {
                            Some(Override::Inherit)  => {
                                quote_spanned! { v.span() =>
                                    [#value] => Some(Self::#name(<#t as core::default::Default>::default())),
                                }
                            }
                            Some(Override::Explicit(default)) => {
                                let default = syn::parse_str::<Path>(default).expect("Path to function");
                                quote_spanned! { v.span() =>
                                    [#value] => Some(Self::#name(#default ())),
                                }
                            },
                            None => {
                                quote! {}
                            }
                        };
                        quote_spanned! { v.span() =>
                                #default
                                [#value, rest@..] => #t::parse_path(rest).map(Self::#name)
                        }
                    }
                    None => {
                        quote_spanned! { v.span() =>
                            [#value] => Some(Self::#name()),
                        }
                    }
                }
            }
            // variables
            Fields::Named(variables) => {
                let vars = variables
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().expect("Tuples are not supported"));

                let values = variables.named.iter().map(|f| {
                    let ident = f.ident.as_ref().expect("Tuples are not supported");

                    quote! {
                        #ident: (*#ident).into()
                    }
                });

                quote_spanned! { v.span() =>
                    [#value, #(#vars),*] => Some(Self::#name{ #(#values),* })
                }
            }
        }
    });

    let mappers = data.variants.iter().map(|v| {
        let name = &v.ident;

        let fn_base_name = name.to_string().to_case(Case::Snake);

        let map_name = format_ident!("map_{}", fn_base_name);
        let mapper_name = format_ident!("mapper_{}", fn_base_name);
        let with_name = format_ident!("with_{}", fn_base_name);

        match &v.fields {
            Fields::Unit => quote!(),
            Fields::Unnamed(fields) => {
                let tys = fields
                    .unnamed
                    .iter()
                    .map(|f| f.ty.clone())
                    .collect::<Vec<_>>();

                let vars = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("arg_{i}"))
                    .collect::<Vec<_>>();

                let tys = match tys.len() {
                    1 => {
                        let f = tys.first().unwrap();
                        quote! { #f }
                    }
                    _ => {
                        quote! {(#(#tys),*)}
                    }
                };

                let vars = match vars.len() {
                    1 => {
                        let f = vars.first().unwrap();
                        quote! { #f }
                    }
                    _ => {
                        quote! {(#(#vars),*)}
                    }
                };

                quote!(
                    pub fn #map_name(self) -> Option<#tys> {
                        match self {
                            Self::#name(#vars) => Some(#vars),
                            _ => None,
                        }
                    }

                    pub fn #mapper_name(_:()) -> yew_nested_router::prelude::Mapper<Self, #tys> {
                        (Self::#map_name, Self::#name).into()
                    }

                    pub fn #with_name<F, R>(f: F) -> impl Fn(Self) -> R
                    where
                        F: Fn(#tys) -> R,
                        R: std::default::Default
                    {
                        move |s| s.#map_name().map(|v|f(v)).unwrap_or_default()
                    }
                )
            }
            Fields::Named(fields) => {
                let tys = fields
                    .named
                    .iter()
                    .map(|f| f.ty.clone())
                    .collect::<Vec<_>>();

                let vars = fields
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref())
                    .collect::<Vec<_>>();
                quote!(
                    pub fn #map_name(self) -> Option<(#(#tys),*)> {
                        match self {
                            Self::#name{#(#vars),*} => Some((#(#vars),*)),
                            _ => None,
                        }
                    }
                )
            }
        }
    });

    let predicates = data.variants.iter().map(|v| {
        let name = &v.ident;

        let fn_name = name.to_string().to_case(Case::Snake);
        let fn_name = format_ident!("is_{}", fn_name);

        match &v.fields {
            Fields::Unit => quote!(
                pub fn #fn_name(self) -> bool {
                    matches!(self, Self::#name)
                }
            ),
            Fields::Unnamed(_) => {
                quote!(
                    pub fn #fn_name(self) -> bool {
                        matches!(self, Self::#name(_))
                    }
                )
            }
            Fields::Named(_) => {
                quote!(
                    pub fn #fn_name(self) -> bool {
                        matches!(self, Self::#name{..})
                    }
                )
            }
        }
    });

    let output = quote! {
        impl yew_nested_router::target::Target for #ident {

                fn render_self_into(&self, path: &mut Vec<String>) {
                    match self {
                        #(#render_self ,)*
                    }
                }

                fn render_path_into(&self, path: &mut Vec<String>) {
                    match self {
                        #(#render_path ,)*
                    }
                }

                fn parse_path(path: &[&str]) -> Option<Self> {
                    match path {
                        #(#parse_path ,)*
                        _ => None,
                    }
                }

        }

        impl #ident {
            #(#mappers)*
            #(#predicates)*

            pub fn any(self) -> bool { true }
        }
    };

    output.into()
}
