extern crate core;

use convert_case::{Case, Casing};
use darling::util::{Flag, Override};
use darling::{FromField, FromVariant};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DataEnum, DeriveInput,
    Field, Fields, Path, Token, Variant,
};

/// Get the value of the path segment
fn to_discriminator(variant: &Variant, opts: &Opts) -> String {
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
}

#[derive(FromField, Default)]
#[darling(default, attributes(target))]
struct FieldOpts {
    nested: Flag,
    value: Flag,
    default: Option<Override<String>>,
}

impl FieldOpts {
    fn validate(self) -> Self {
        if self.nested.is_present() && self.value.is_present() {
            panic!("Cannot configure a field as both 'nested' and 'value'");
        }
        self
    }
}

/// Find the fields which points to the nested target.
fn nested_field<P>(
    expect_target: bool,
    fields: &Punctuated<Field, P>,
) -> (Vec<&Field>, Option<&Field>) {
    let mut values = vec![];

    for (i, field) in fields.iter().enumerate() {
        let opts = FieldOpts::from_field(field)
            .expect("Unable to parse field options")
            .validate();

        let last = i == fields.len() - 1;

        if last {
            if (expect_target || opts.nested.is_present()) && !opts.value.is_present() {
                // this is the last field, and it is flagged as nested, we can return now
                return (values, Some(field));
            }
        } else {
            if opts.nested.is_present() {
                panic!(
                    "Only the last field can be a nested target: {}",
                    field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| format!("{}", i))
                );
            }
        }

        values.push(field);
    }

    (values, None)
}

/// render the full path, this needs to dive into nested entries.
fn render_path(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        match &v.fields {
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    Self::#name => {
                        self.render_self_into(__internal_path);
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let (values, nested) = nested_field(true, &fields.unnamed);

                // expand all values to captures of _, expect if the last one is "nested".
                let values = values
                    .iter()
                    .map(|_| quote!(_))
                    .chain(nested.map(|_| quote!(nested)));

                let nested = match nested.is_some() {
                    true => {
                        quote! { nested.render_path_into(__internal_path); }
                    }
                    false => {
                        quote! {}
                    }
                };

                quote_spanned! { v.span() =>
                    Self::#name(#(#values),*) => {
                        self.render_self_into(__internal_path);
                        #nested
                    }
                }
            }
            Fields::Named(fields) => {
                // we capture the nested field as "nested" and then call it
                let (capture, nested) = match nested_field(false, &fields.named) {
                    (_, Some(nested)) => {
                        let nested = nested.ident.as_ref().expect("Field must have a name");
                        (
                            quote! { #nested: nested, .. },
                            quote! { nested.render_path_into(__internal_path); },
                        )
                    }
                    (_, None) => (quote! {..}, quote! {}),
                };

                quote_spanned! { v.span() =>
                    Self::#name{ #capture } => {
                        self.render_self_into(__internal_path);
                        #nested
                    }
                }
            }
        }
    })
}

/// generate iterators for capturing and rendering values
fn capture_values<'f, P, F>(
    expect_target: bool,
    fields: &'f Punctuated<Field, P>,
    skip_nested: TokenStream,
    push: F,
) -> (
    impl Iterator<Item = TokenStream> + 'f,
    impl Iterator<Item = TokenStream> + 'f,
)
where
    F: Fn(&Ident) -> TokenStream + 'static,
{
    let (values, nested) = nested_field(expect_target, fields);

    let captures = values
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, f)| {
            let anon = format_ident!("arg_{}", i);
            let name = f.ident.as_ref().unwrap_or(&anon);
            quote! { #name }
        })
        .chain(nested.map(|_| skip_nested));

    let values = values.into_iter().enumerate().map(move |(i, f)| {
        let anon = format_ident!("arg_{}", i);
        let name = f.ident.as_ref().unwrap_or(&anon);
        push(name)
    });

    (captures, values)
}

/// rendering (local) target to its path.
fn render_self(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        let opts = Opts::from_variant(v).expect("Unable to parse options");
        let disc = to_discriminator(&v, &opts);

        match &v.fields {
            // plain route
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    Self::#name => { __internal_path.push(#disc.into()); }
                }
            }
            // nested route
            Fields::Unnamed(fields) => {
                let (captures, values) =
                    capture_values(true, &fields.unnamed, quote! { _ }, |name| {
                        quote! { __internal_path.push(#name.to_string()); }
                    });

                quote_spanned! { v.span() =>
                    Self::#name(#(#captures),*) => {
                        __internal_path.push(#disc.into());
                        #(#values)*
                    }
                }
            }
            // variables
            Fields::Named(fields) => {
                let (captures, values) =
                    capture_values(false, &fields.named, quote! { .. }, |name| {
                        quote! { __internal_path.push(#name.to_string()); }
                    });

                quote_spanned! { v.span() =>
                    Self::#name { #(#captures),* } => {
                        __internal_path.push(#disc.into());
                        #(#values)*
                    }
                }
            }
        }
    })
}

/// parsing the path, into a target
fn parse_path(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        let opts = Opts::from_variant(v).expect("Unable to parse options");
        let value = to_discriminator(&v, &opts);

        match &v.fields {
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    [#value] => Some(Self::#name)
                }
            }
            Fields::Unnamed(fields) => parse_rules(
                &v,
                true,
                &fields.unnamed,
                |_, cap| from_str(&cap),
                |_, target| quote!(#target),
                |name, values, target| quote!(Some(Self::#name(#(#values, )* #target))),
            ),
            Fields::Named(fields) => parse_rules(
                &v,
                false,
                &fields.named,
                |name, cap| {
                    let name = name.expect("Must have a name");
                    let from = from_str(&cap);
                    quote!(#name: #from)
                },
                |name, target| {
                    let name = name.expect("Must have a name");
                    quote!(#name: #target)
                },
                |name, values, target| quote!(Some(Self::#name { #(#values, )* #target})),
            ),
        }
    })
}

fn parse_rules<P, F1, F2, F3>(
    v: &Variant,
    expect_target: bool,
    fields: &Punctuated<Field, P>,
    converter: F1,
    target_converter: F2,
    ctor: F3,
) -> TokenStream
where
    F1: Fn(Option<&Ident>, &Ident) -> TokenStream,
    F2: Fn(Option<&Ident>, TokenStream) -> TokenStream,
    F3: Fn(&Ident, &Vec<TokenStream>, TokenStream) -> TokenStream,
{
    let name = &v.ident;

    let (values, nested) = nested_field(expect_target, fields);

    let opts = Opts::from_variant(v).expect("Unable to parse options");
    let disc = to_discriminator(&v, &opts);

    let (captures, values): (Vec<_>, Vec<_>) = values
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = f.ident.as_ref();
            let cap = f
                .ident
                .as_ref()
                .map(|f| format_ident!("value_{f}"))
                .unwrap_or_else(|| format_ident!("arg_{i}"));
            (quote!(#cap), converter(name, &cap))
        })
        .unzip();

    match nested {
        Some(nested) => {
            let t = &nested.ty;

            let field_opts = FieldOpts::from_field(&nested).expect("Unable to parse field options");

            let default = match &field_opts.default {
                Some(Override::Inherit) => {
                    let init = ctor(
                        name,
                        &values,
                        target_converter(
                            nested.ident.as_ref(),
                            quote!(<#t as core::default::Default>::default()),
                        ),
                    );
                    quote! {
                        [#disc, #(#captures, )*] => #init,
                    }
                }
                Some(Override::Explicit(default)) => {
                    let default = syn::parse_str::<Path>(default).expect("Path to function");
                    let init = ctor(
                        name,
                        &values,
                        target_converter(nested.ident.as_ref(), quote!(#default ())),
                    );
                    quote! {
                        [#disc, #(#captures, )*] => #init,
                    }
                }
                None => {
                    quote! {}
                }
            };

            let init = ctor(
                name,
                &values,
                target_converter(nested.ident.as_ref(), quote!(target)),
            );
            quote_spanned! { v.span() =>
                #default
                [#disc, #(#captures, )* rest@..] => match #t::parse_path(rest) {
                    Some(target) => #init,
                    None => None,
                }
            }
        }
        None => {
            let init = ctor(name, &values, quote!());
            quote_spanned! { v.span() =>
                [#disc, #(#captures),*] => #init
            }
        }
    }
}

fn from_str(cap: &Ident) -> TokenStream {
    quote!({
        match std::str::FromStr::from_str(#cap) {
            Ok(v) => v,
            Err(err) => return None,
        }
    })
}

/// Mapping of variants to its values.
fn mappers(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        let fn_base_name = name.to_string().to_case(Case::Snake);

        let map_name = format_ident!("map_{}", fn_base_name);
        let mapper_name = format_ident!("mapper_{}", fn_base_name);
        let with_name = format_ident!("with_{}", fn_base_name);

        let none = Punctuated::<Field, Token![,]>::new();
        let fields = match &v.fields {
            Fields::Unnamed(fields) => fields.unnamed.iter(),
            Fields::Named(fields) => fields.named.iter(),
            Fields::Unit => none.iter(),
        };

        let (captures, types): (Vec<_>, Vec<_>) = fields
            .enumerate()
            .map(|(i, f)| {
                let cap = f.ident.clone().unwrap_or_else(|| format_ident!("arg_{i}"));
                let ty = &f.ty;
                (quote!(#cap), quote!(#ty))
            })
            .unzip();

        let (types, init) = match types.len() {
            1 => {
                let t = types.first().unwrap();
                let c = captures.first().unwrap();
                (quote!(#t), quote!(#c))
            }
            _ => (quote!((#(#types),*)), quote!((#(#captures),*))),
        };

        match &v.fields {
            Fields::Unit => quote_spanned! { v.span() => },
            Fields::Unnamed(fields) => {
                let (_, nested ) = nested_field(true, &fields.unnamed);

                let mapper = match nested {
                    Some(_) => {
                        quote!{
                            #[allow(unused)]
                            pub fn #mapper_name(_:()) -> yew_nested_router::prelude::Mapper<Self, #types> {
                                (Self::#map_name, Self::#name).into()
                            }
                        }
                    },
                    None => quote!(),
                };

                quote_spanned! { v.span() =>
                    #[allow(unused)]
                    pub fn #map_name(self) -> Option<#types> {
                        match self {
                            Self::#name(#(#captures),*) => Some(#init),
                            _ => None,
                        }
                    }

                    #mapper

                    #[allow(unused)]
                    pub fn #with_name<F, R>(f: F) -> impl Fn(Self) -> R
                    where
                        F: Fn(#types) -> R,
                        R: std::default::Default
                    {
                        move |s| s.#map_name().map(&f).unwrap_or_default()
                    }
                }
            },
            Fields::Named(_) => quote_spanned! { v.span() =>
                #[allow(unused)]
                pub fn #map_name(self) -> Option<#types> {
                    match self {
                        Self::#name{#(#captures),*} => Some(#init),
                        _ => None,
                    }
                }

                #[allow(unused)]
                pub fn #with_name<F, R>(f: F) -> impl Fn(Self) -> R
                where
                    F: Fn(#types) -> R,
                    R: std::default::Default
                {
                    move |s| s.#map_name().map(&f).unwrap_or_default()
                }
            },
        }
    })
}

/// create `is_<variant>` functions, which check if the instance is matches the variant, ignoring
/// additional values.
fn predicates(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        let fn_name = name.to_string().to_case(Case::Snake);
        let fn_name = format_ident!("is_{}", fn_name);

        match &v.fields {
            Fields::Unit => quote_spanned! { v.span() =>
                #[allow(unused)]
                pub fn #fn_name(self) -> bool {
                    matches!(self, Self::#name)
                }
            },
            Fields::Unnamed(fields) => {
                let captures = fields.unnamed.iter().map(|_| quote! {_});
                quote_spanned! { v.span() =>
                    #[allow(unused)]
                    pub fn #fn_name(self) -> bool {
                        matches!(self, Self::#name( #(#captures),* ))
                    }
                }
            }
            Fields::Named(_) => {
                quote_spanned! { v.span() =>
                    #[allow(unused)]
                    pub fn #fn_name(&self) -> bool {
                        matches!(*self, Self::#name{..})
                    }
                }
            }
        }
    })
}

/// Helps implementing the `Target` trait in an enum.
#[proc_macro_derive(Target, attributes(target))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let data = match data {
        Data::Enum(e) => e,
        _ => panic!("Derive must be used on enum only"),
    };

    let render_path = render_path(&data);
    let render_self = render_self(&data);
    let parse_path = parse_path(&data);
    let mappers = mappers(&data);
    let predicates = predicates(&data);

    let output = quote! {
        impl yew_nested_router::target::Target for #ident {

                fn render_self_into(&self, __internal_path: &mut Vec<String>) {
                    match self {
                        #(#render_self ,)*
                    }
                }

                fn render_path_into(&self, __internal_path: &mut Vec<String>) {
                    match self {
                        #(#render_path ,)*
                    }
                }

                fn parse_path(__internal_path: &[&str]) -> Option<Self> {
                    match __internal_path {
                        #(#parse_path ,)*
                        _ => None,
                    }
                }

        }

        impl #ident {
            #(#mappers)*
            #(#predicates)*

            #[inline]
            pub fn any(self) -> bool { true }
        }
    };

    output.into()
}
