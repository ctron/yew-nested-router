use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;
use yew_nested_router::prelude::parameter_value::ParameterValue;
use yew_nested_router::target::Target;
use yew_nested_router::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationContext {
    Any,
}

impl FromStr for ApplicationContext {
    type Err = ();

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self::Any)
    }
}

impl Display for ApplicationContext {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Target, Debug, Clone, PartialEq, Eq)]
pub enum DetailsSection {
    Yaml,
    Debug,
    #[target(index)]
    Overview,
}

#[test]
fn test1() {
    #[derive(Target, Debug, Clone, PartialEq, Eq)]
    pub enum Pages {
        Details {
            app: ApplicationContext,
            name: String,
            #[target(nested)]
            details: DetailsSection,
        },
        Index {
            app: ApplicationContext,
        },
    }

    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name", "debug"], &[]),
        Some(Pages::Details {
            details: DetailsSection::Debug,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    )
}

/// test with a nested struct like variant having a default.
#[test]
fn test_default_inherit() {
    #[derive(Target, Debug, Clone, PartialEq, Eq)]
    pub enum Pages {
        Details {
            app: ApplicationContext,
            name: String,
            #[target(nested, default)]
            details: DetailsSection,
        },
        #[allow(unused_variables)]
        Index { app: ApplicationContext },
    }

    impl Default for DetailsSection {
        fn default() -> Self {
            Self::Overview
        }
    }

    // defaults to "overview"
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name"], &[]),
        Some(Pages::Details {
            details: DetailsSection::Overview,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );

    // override still works
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name", "debug"], &[]),
        Some(Pages::Details {
            details: DetailsSection::Debug,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );
}

/// test with a nested struct like variant having a default.
#[test]
fn test_default_explicit() {
    #[derive(Target, Debug, Clone, PartialEq, Eq)]
    pub enum Pages {
        Details {
            app: ApplicationContext,
            name: String,
            #[target(nested, default = "default_section")]
            details: DetailsSection,
        },
        Index {
            app: ApplicationContext,
        },
    }

    fn default_section() -> DetailsSection {
        DetailsSection::Yaml
    }

    // defaults to "yaml"
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name"], &[]),
        Some(Pages::Details {
            details: DetailsSection::Yaml,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );

    // override still works
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name", "debug"], &[]),
        Some(Pages::Details {
            details: DetailsSection::Debug,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );
}
#[test]
fn test_param() {
    #[derive(Target, Debug, Clone, PartialEq, Eq)]
    pub enum Pages {
        Details {
            app: ApplicationContext,
            name: String,
            #[target(query)]
            param: String,
            #[target(nested)]
            details: DetailsSection,
        },
        Index {
            app: ApplicationContext,
        },
    }
}

#[test]
fn test_url_parser() {
    #[derive(Target, Debug, Clone, PartialEq, Eq)]
    pub enum Pages {
        Details {
            app: ApplicationContext,
            name: String,
            #[target(nested)]
            details: DetailsSection,
        },
        Index {
            app: ApplicationContext,
        },
    }
    assert_eq!(
        Pages::parse_url(
            "http://localhost:4200/my-ui/",
            "http://localhost:4200/my-ui/details/my-app/my-name/debug",
        ),
        Some(Pages::Details {
            details: DetailsSection::Debug,
            name: "my-name".to_string(),
            app: ApplicationContext::Any,
        })
    )
}

#[test]
fn test_param_manual() {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Pages {
        Details {
            name: String,
            param: String,
            number: f32,
            details: DetailsSection,
        },
    }
    impl Target for Pages {
        fn render_self_into<'a>(
            &'a self,
            __internal_path: &mut Vec<Cow<'a, str>>,
            _params: &mut Vec<(Cow<'a, str>, Cow<'a, str>)>,
        ) {
            match self {
                Self::Details {
                    name,
                    param,
                    number,
                    ..
                } => {
                    __internal_path.push("details".into());
                    __internal_path.push(name.to_string().into());
                    for value in param.to_parameter_values().iter() {
                        _params.push(("param".into(), value.to_string().into()));
                    }
                    for value in number.to_parameter_values().iter() {
                        _params.push(("number".into(), value.to_string().into()));
                    }
                }
            }
        }
        fn render_path_into<'a>(
            &'a self,
            __internal_path: &mut Vec<Cow<'a, str>>,
            _params: &mut Vec<(Cow<'a, str>, Cow<'a, str>)>,
        ) {
            match self {
                Self::Details {
                    details: nested,
                    param,
                    ..
                } => {
                    self.render_self_into(__internal_path, _params);
                    nested.render_path_into(__internal_path, _params);
                }
            }
        }
        fn parse_path(
            __internal_path: &[&str],
            __query_params: &[(std::borrow::Cow<str>, std::borrow::Cow<str>)],
        ) -> Option<Self> {
            match __internal_path {
                ["details", value_name, rest @ ..] => {
                    if let (Some(details), Some(param), Some(number), Ok(name)) = (
                        DetailsSection::parse_path(rest, __query_params),
                        String::extract_from_params(__query_params, "param"),
                        f32::extract_from_params(__query_params, "number"),
                        std::str::FromStr::from_str(value_name),
                    ) {
                        Some(Self::Details {
                            name,
                            param,
                            number,
                            details,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
    assert_eq!(
        Pages::parse_url(
            "http://localhost:4200/my-ui/",
            "http://localhost:4200/my-ui/details/my-name/debug?param=hello&number=2.3",
        ),
        Some(Pages::Details {
            details: DetailsSection::Debug,
            name: "my-name".to_string(),
            param: "hello".to_string(),
            number: 2.3,
        })
    );
    let mut url = Url::parse("http://localhost:4200/my-ui/").unwrap();
    Pages::Details {
        details: DetailsSection::Debug,
        name: "my-name".to_string(),
        param: "hello".to_string(),
        number: 0.0,
    }
    .append_url(&mut url)
    .unwrap();
    assert_eq!(
        url.to_string(),
        "http://localhost:4200/my-ui/details/my-name/debug?param=hello&number=0"
    );
}
