extern crate core;

use darling::util::Override;
use darling::FromVariant;
use proc_macro::{self, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Path, Variant};

/// Get the value of the path segment
fn to_value(variant: &Variant, opts: &Opts) -> String {
    opts.rename
        .clone()
        .unwrap_or_else(|| variant.ident.to_string().to_lowercase())
}

#[derive(FromVariant, Default)]
#[darling(default, attributes(target))]
struct Opts {
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
    };
    output.into()
}
