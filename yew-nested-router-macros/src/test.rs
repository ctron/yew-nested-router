use quote::{quote, ToTokens};
use syn::parse2;
use syn::{parse_quote, Data, DataEnum, ItemEnum};

#[test]
fn test_simple() {
    let input: ItemEnum = parse_quote! {
            #[derive(Target, Debug, Clone, PartialEq, Eq)]
            pub enum Pages {
                Details {
                    app: ApplicationContext,
                    name: String,
                    #[target(query)]
                    param: String,
                    #[target(nested, default = "default_section")]
                    details: DetailsSection,
                },
                Index(ApplicationContext),
            }
    };
    generate_enum(parse_quote!(
        #[derive(Clone, Debug, PartialEq, Eq, Target, Default)]
        pub enum Details {
            #[default]
            D1,
            D2,
        }
    ));

    generate_enum(parse_quote! {
     #[derive(Clone, Debug, PartialEq, Target)]
     pub enum AppRoute {
       #[target(index)]
       Index,
       Foo(#[target(default)] Details),
       Bar {
         id: String,
         #[target(nested, default)]
         details: Details,
       },
             Params{
                 #[target(query)]
                 number1: f32,
                 #[target(query)]
                 number2: u8,
             }
     }
    });
}

fn generate_enum(input: ItemEnum) {
    let mut token_stream = input.to_token_stream();
    let additional_tokens = crate::render_enum(
        input.ident,
        Data::Enum(DataEnum {
            enum_token: input.enum_token,
            brace_token: input.brace_token,
            variants: input.variants,
        }),
    );

    token_stream.extend(quote! {
        #[doc=" --------------------------- Generated code from here ---------------------------"]
    });
    token_stream.extend(additional_tokens.to_token_stream());

    let result = parse2(token_stream.clone());
    match result {
        Ok(f) => {
            println!("{}", prettyplease::unparse(&f));
        }
        Err(e) => {
            println!("Invalid code:\n{token_stream}");
            panic!("Cannot parse token stream: {e}")
        }
    }
}
