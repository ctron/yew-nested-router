//! A router for Yew, supporting nesting.

pub mod components;

mod navigation;
mod nested;
mod router;
mod switch;
mod target;

pub use nested::Nested;
pub use router::Router;
pub use switch::Switch;
pub use target::Target;

/// Common includes.
pub mod prelude {
    pub use super::nested::*;
    pub use super::router::*;
    pub use super::switch::*;
    pub use super::target::*;
}
