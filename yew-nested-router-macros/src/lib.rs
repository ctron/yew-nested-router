extern crate core;

use convert_case::{Case, Casing};
use darling::{
    util::{Flag, Override},
    FromField, FromVariant,
};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DataEnum, DeriveInput,
    Field, Fields, Token, Variant,
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

#[derive(FromVariant, Default, Debug)]
#[darling(default, attributes(target))]
struct Opts {
    index: Flag,
    rename: Option<String>,
}

#[derive(FromField, Default, Debug)]
#[darling(default, attributes(target))]
struct FieldOpts {
    nested: Flag,
    value: Flag,
    default: Option<Override<String>>,
    query: Option<Override<String>>,
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
) -> (Vec<&Field>, Option<&Field>, Vec<(&Field, Literal)>) {
    let mut values = vec![];
    let mut query_params = vec![];

    for (i, field) in fields.iter().enumerate() {
        let opts = FieldOpts::from_field(field)
            .expect("Unable to parse field options")
            .validate();

        let last = i == fields.len() - 1;

        if opts.query.is_some() && opts.nested.is_present() {
            panic!(
                "A query parameter cannot be nested: {}",
                field
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("{}", i))
            );
        }
        if let Some(query) = opts.query {
            query_params.push((
                field,
                match query {
                    Override::Inherit => {
                        let ident = field
                            .ident
                            .as_ref()
                            .expect("anyonymous query parameters needs explizit name");
                        let mut l = Literal::string(ident.to_string().as_ref());
                        l.set_span(field.span());
                        l
                    }
                    Override::Explicit(name) => Literal::string(&name),
                },
            ));
        } else {
            if last {
                if (expect_target || opts.nested.is_present()) && !opts.value.is_present() {
                    // this is the last field, and it is flagged as nested, we can return now
                    return (values, Some(field), query_params);
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if opts.nested.is_present() {
                    panic!(
                        "Only the last field can be a nested target: {}",
                        field
                            .ident
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| format!("{}", i))
                    );
                }
            }

            values.push(field);
        }
    }

    (values, None, query_params)
}

/// render the full path, this needs to dive into nested entries.
fn render_path(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        match &v.fields {
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    Self::#name => {
                        self.render_self_into(__internal_path,__params);
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let (values, nested, query_params) = nested_field(true, &fields.unnamed);
                if !query_params.is_empty() {
                    panic!("No anonymous query parameters supported")
                }

                // expand all values to captures of _, expect if the last one is "nested".
                let values = values
                    .iter()
                    .map(|_| quote!(_))
                    .chain(nested.map(|_| quote!(nested)));

                let nested = match nested.is_some() {
                    true => {
                        quote! { nested.render_path_into(__internal_path,__params); }
                    }
                    false => {
                        quote! {}
                    }
                };

                quote_spanned! { v.span() =>
                    Self::#name(#(#values),*) => {
                        self.render_self_into(__internal_path,__params);
                        #nested
                    }
                }
            }
            Fields::Named(fields) => {
                // we capture the nested field as "nested" and then call it
                let (capture, nested) = match nested_field(false, &fields.named) {
                    (_, Some(nested), _) => {
                        let nested = nested.ident.as_ref().expect("Field must have a name");
                        (
                            quote! { #nested: nested, .. },
                            quote! { nested.render_path_into(__internal_path, __params); },
                        )
                    }
                    (_, None, _) => (quote! {..}, quote! {}),
                };

                quote_spanned! { v.span() =>
                    Self::#name{ #capture } => {
                        self.render_self_into(__internal_path,__params);
                        #nested
                    }
                }
            }
        }
    })
}

/// generate iterators for capturing and rendering values
fn capture_values<P, F>(
    expect_target: bool,
    fields: &Punctuated<Field, P>,
    skip_nested: TokenStream,
    push: F,
) -> (
    impl Iterator<Item = TokenStream> + '_,
    impl Iterator<Item = TokenStream> + '_,
)
where
    F: Fn(&Ident) -> TokenStream + 'static,
{
    let (values, nested, query_params) = nested_field(expect_target, fields);

    let captures = values
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, f)| {
            let anon = format_ident!("arg_{}", i);
            let name = f.ident.as_ref().unwrap_or(&anon);
            quote! { #name }
        })
        .chain(
            query_params
                .clone()
                .into_iter()
                .enumerate()
                .map(|(idx, (f, _))| {
                    let alias = format_ident!("p{idx}");
                    if let Some(name) = f.ident.as_ref() {
                        quote! {#name: #alias}
                    } else {
                        quote! {#alias}
                    }
                }),
        )
        .chain(nested.map(|_| skip_nested));

    let values = values
        .into_iter()
        .enumerate()
        .map(move |(i, f)| {
            let anon = format_ident!("arg_{}", i);
            let name = f.ident.as_ref().unwrap_or(&anon);
            push(name)
        })
        .chain(
            query_params
                .into_iter()
                .enumerate()
                .map(|(idx, (_, name))| push_params(&name, &format_ident!("p{idx}"))),
        );
    (captures, values)
}

fn push_params(name: &Literal, alias: &Ident) -> TokenStream {
    quote! {__params.push((#name.into(),yew_nested_router::prelude::parameter_value::ParameterValue::to_parameter_value(#alias)));}
}

/// rendering (local) target to its path.
fn render_self(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|v| {
        let name = &v.ident;

        let opts = Opts::from_variant(v).expect("Unable to parse options");
        let disc = to_discriminator(v, &opts);

        match &v.fields {
            // plain route
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    Self::#name => { __internal_path.push(#disc.to_string().into()); }
                }
            }
            // nested route
            Fields::Unnamed(fields) => {
                let (captures, values) =
                    capture_values(true, &fields.unnamed, quote! { _ }, |name| {
                        quote! { __internal_path.push(#name.to_string().into()); }
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
                        quote! { __internal_path.push(#name.to_string().into()); }
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
        let value = to_discriminator(v, &opts);

        match &v.fields {
            Fields::Unit => {
                quote_spanned! { v.span() =>
                    [#value] => Some(Self::#name)
                }
            }
            Fields::Unnamed(fields) => parse_rules(
                v,
                true,
                &fields.unnamed,
                |_, cap| from_str_expr(cap),
                |_, target| quote!(#target),
                |name, values| quote!(Self::#name(#(#values, )*)),
            ),
            Fields::Named(fields) => parse_rules(
                v,
                false,
                &fields.named,
                |name, cap| {
                    let name = name.expect("Must have a name");
                    let from = from_str_expr(cap);
                    quote!(#name: #from)
                },
                |name, target| {
                    let name = name.expect("Must have a name");
                    quote!(#name: #target)
                },
                |name, values| quote!(Self::#name { #(#values, )*}),
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
    F3: Fn(&Ident, &Vec<TokenStream>) -> TokenStream,
{
    let name = &v.ident;

    let (values, nested, query_params) = nested_field(expect_target, fields);

    let opts = Opts::from_variant(v).expect("Unable to parse options");
    let disc = to_discriminator(v, &opts);

    let mut captures: Vec<TokenStream> = vec![];
    let mut let_matches: Vec<TokenStream> = vec![];
    let mut patterns: Vec<TokenStream> = vec![];
    let mut init: Vec<TokenStream> = vec![];

    for (i, f) in values.iter().enumerate() {
        let argument = format_ident!("arg_{i}");
        patterns.push(from_str_expr(&argument));
        let_matches.push(quote! {Ok(#argument)});
        init.push(if let Some(name) = f.ident.as_ref() {
            quote! {#name: #argument}
        } else {
            argument.clone().to_token_stream()
        });
        captures.push(argument.into_token_stream());
    }
    for (idx, (field, parameter_name)) in query_params.into_iter().enumerate() {
        let argument = format_ident!("param_{idx}");
        patterns.push(quote! {yew_nested_router::target::parameter_value::ParameterValue::extract_from_params(__query_params,#parameter_name)});
        let_matches.push(quote! {Some(#argument)});
        init.push(if let Some(name) = field.ident.as_ref() {
            quote! {#name: #argument}
        } else {
            argument.clone().to_token_stream()
        });
    }
    let default = if let Some(nested) = nested {
        let t = &nested.ty;

        let field_opts = FieldOpts::from_field(nested).expect("Unable to parse field options");
        let default_branch = if let Some(default) = &field_opts.default {
            let default_value = match default {
                Override::Inherit => {
                    quote! {Default::default()}
                }
                Override::Explicit(initializer) => {
                    let method_name = Ident::new(initializer, nested.span());
                    quote! {#method_name()}
                }
            };
            let nested_default_initializer = nested
                .ident
                .as_ref()
                .map(|ident| quote! {#ident: #default_value})
                .unwrap_or(quote! {#default_value});
            let mut init = init.clone();
            init.push(nested_default_initializer);
            let initializer = ctor(name, &init);

            Some(quote_spanned! { v.span() =>
                #[allow(unused_parens,irrefutable_let_patterns)]
                [#disc, #(#captures, )*] => if let (#(#let_matches),*)=(#(#patterns),*) {
                    Some(#initializer)
                }else{
                    None
                }
            })
        } else {
            None
        };
        let_matches.push(quote! {Some(nested)});
        patterns.push(quote! {#t::parse_path(rest, __query_params)});
        init.push(
            nested
                .ident
                .as_ref()
                .map(|ident| quote! {#ident: nested})
                .unwrap_or(quote! {nested}),
        );
        captures.push(quote! {rest@..});
        default_branch
    } else {
        None
    };
    let initializer = ctor(name, &init);

    quote_spanned! { v.span() =>
        #default
        #[allow(unused_parens,irrefutable_let_patterns)]
        [#disc, #(#captures, )*] => if let (#(#let_matches),*)=(#(#patterns),*) {
            Some(#initializer)
        }else{
            None
        }
    }
}

fn from_str_expr(cap: &Ident) -> TokenStream {
    quote! {std::str::FromStr::from_str(#cap)}
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
                let (_, nested, _) = nested_field(true, &fields.unnamed);

                let mapper = match nested {
                    Some(_) => {
                        quote! {
                            #[allow(unused)]
                            pub fn #mapper_name(_:()) -> yew_nested_router::prelude::Mapper<Self, #types> {
                                (Self::#map_name, Self::#name).into()
                            }
                        }
                    }
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
            }
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
                #[allow(clippy::wrong_self_convention)]
                pub fn #fn_name(self) -> bool {
                    matches!(self, Self::#name)
                }
            },
            Fields::Unnamed(_) => {
                quote_spanned! { v.span() =>
                    #[allow(unused)]
                    #[allow(clippy::wrong_self_convention)]
                    pub fn #fn_name(self) -> bool {
                        matches!(self, Self::#name( .. ))
                    }
                }
            }
            Fields::Named(_) => {
                quote_spanned! { v.span() =>
                    #[allow(unused)]
                    #[allow(clippy::wrong_self_convention)]
                    pub fn #fn_name(self) -> bool {
                        matches!(self, Self::#name{..})
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

    render_enum(ident, data).into()
}

fn render_enum(ident: Ident, data: Data) -> TokenStream {
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

                fn render_self_into<'a>(&'a self, __internal_path: &mut Vec<std::borrow::Cow<'a, str>>, __params: &mut Vec<(std::borrow::Cow<'a, str>,std::borrow::Cow<'a, str>)>) {
                    match self {
                        #(#render_self ,)*
                    }
                }

                fn render_path_into<'a>(&'a self, __internal_path: &mut Vec<std::borrow::Cow<'a, str>>, __params: &mut Vec<(std::borrow::Cow<'a, str>,std::borrow::Cow<'a, str>)>) {
                    match self {
                        #(#render_path ,)*
                    }
                }

                fn parse_path(__internal_path: &[&str],__query_params: &[(std::borrow::Cow<str>,std::borrow::Cow<str>)]) -> Option<Self> {
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
    output
}

#[cfg(test)]
mod test;
