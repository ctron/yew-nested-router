use yew_nested_router::Target;

#[derive(Clone, Debug, PartialEq, Eq, Target)]
pub enum Page {
    /// Simple example
    #[target(index)]
    Index,
    /// Simple example
    A,
    /// Nested target example
    #[target(default)]
    B(B),
    /// Target with data
    C { value: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Target)]
pub enum B {
    #[default]
    #[target(rename = "eins")]
    One,
    #[target(rename = "deux")]
    Two(View),
    Three(View),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Target)]
pub enum View {
    Overview,
    Details,
    Source,
}
