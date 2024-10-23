use yew_nested_router::prelude::{Mapper, Target};

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
    /// Nested target with custom mapper that captures value
    D {
        id: u32,
        #[target(nested)]
        target: D,
    },
}

impl Page {
    // We will need to pass the id in from a closure when calling this
    pub fn mapper_d(id: u32) -> Mapper<Page, D> {
        let downwards = |page| match page {
            Page::D { target, .. } => Some(target),
            _ => None,
        };
        let upwards = move |d| Page::D { id, target: d };
        Mapper::new(downwards, upwards)
    }
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
        #[target(query)]
        param: String,
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Target)]
pub enum D {
    #[default]
    First,
    Second,
}
