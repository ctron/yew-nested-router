//! A router for Yew, supporting nesting.

pub mod components;
pub mod target;

mod navigation;
mod nested;
mod router;
mod switch;

pub use nested::Nested;
pub use router::Router;
pub use switch::Switch;
pub use yew_nested_router_macros::Target;

/// Common includes.
pub mod prelude {
    pub use super::nested::*;
    pub use super::router::*;
    pub use super::switch::*;
    pub use super::target::*;
}
