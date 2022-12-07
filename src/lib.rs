//! A router for Yew, supporting nesting.

pub mod components;
pub mod target;

mod router;
mod scope;
mod switch;

pub use router::Router;
pub use scope::Scope;
pub use switch::Switch;
pub use yew_nested_router_macros::Target;

/// Common includes.
pub mod prelude {
    pub use super::router::*;
    pub use super::scope::*;
    pub use super::switch::*;
    pub use super::target::*;

    pub use yew_nested_router_macros::Target;
}
