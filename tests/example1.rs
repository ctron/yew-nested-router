use std::fmt::{Display, Formatter};
use std::str::FromStr;
use yew_nested_router::Target;
use yew_nested_router::target::Target;

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
        Pages::parse_path(&["details", "my-app", "my-name", "debug"]),
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
        Index {
            app: ApplicationContext,
        },
    }

    impl Default for DetailsSection {
        fn default() -> Self {
            Self::Overview
        }
    }

    // defaults to "overview"
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name"]),
        Some(Pages::Details {
            details: DetailsSection::Overview,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );

    // override still works
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name", "debug"]),
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
        Pages::parse_path(&["details", "my-app", "my-name"]),
        Some(Pages::Details {
            details: DetailsSection::Yaml,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );

    // override still works
    assert_eq!(
        Pages::parse_path(&["details", "my-app", "my-name", "debug"]),
        Some(Pages::Details {
            details: DetailsSection::Debug,
            name: "my-name".to_string(),
            app: ApplicationContext::Any
        })
    );
}
