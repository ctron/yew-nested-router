use yew_nested_router::prelude::Target;

#[derive(Clone, Debug, PartialEq, Eq, Target)]
pub enum Page {
    /// Simple example
    #[target(index)]
    Index,
    /// Simple example
    A,
    /// Nested target example
    B(#[target(default)] B),
    /// Target with data
    C {
        value: String,
        #[target(nested)]
        target: C,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Target)]
pub enum B {
    #[target(rename = "eins")]
    One,
    #[target(rename = "deux")]
    Two(View),
    Three(View),
}

impl Default for B {
    fn default() -> Self {
        Self::One
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Target)]
pub enum C {
    Foo {
        value: String,
    },
    /// variant with both values being actual values
    Bar(String, #[target(value)] usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Target)]
pub enum View {
    Overview,
    Details,
    Source,
}
