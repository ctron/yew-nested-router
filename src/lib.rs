//! A router for Yew, supporting nesting.
//!
//! ## Usage
//!
//! The nested router makes use of Yew's `context` features. It injects a routing context, tied to
//! a type implementing the [`target::Target`] trait. It then is possible to scope/translate between
//! levels.
//!
//! ### Targets
//!
//! "Targets" are the route targets, things the page can point to. They must be an enum,
//! implementing the [`target::Target`] trait. This can easily be done using the `Target` derive:
//!
//! ```
//! # use yew_nested_router::prelude::*;
//! #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! pub enum AppRoute {
//!   #[target(index)]
//!   Index,
//!   Foo,
//!   Bar,
//! }
//! ```
//!
//! This created a target enum with three paths (`/`, `/foo`, `/bar`).
//!
//! ### Main router
//!
//! Each application needs a main entry point for the router ([`Router`]). This simply injects the
//! routing context, and provides the necessary information internally. All children of the
//! component will simply be rendered.
//!
//! ```
//! # use yew::prelude::*;
//! # use yew_nested_router::prelude::*;
//! # #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! # pub enum AppRoute { Index };
//! # #[function_component(MyContent)] fn my_content() -> Html { html!() }
//! #[function_component(MyApp)]
//! pub fn my_app() -> Html {
//!   html!(
//!     <Router<AppRoute>>
//!       <MyContent/>
//!     </Router<AppRoute>>
//!   )
//! }
//! ```
//!
//! ### Switching content
//!
//! Having the route context available, allows to switch based on its state. This is done using the
//! [`Switch`] component, which searches (upwards) for a matching routing context (of the target
//! type).
//!
//! ```
//! # use yew::prelude::*;
//! # use yew_nested_router::prelude::*;
//! # #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! # pub enum AppRoute {
//! #   #[target(index)]
//! #   Index,
//! #   Foo,
//! #   Bar,
//! # }
//! # #[function_component(Index)] fn index() -> Html { html!() }
//! # #[function_component(Foo)] fn foo() -> Html { html!() }
//! # #[function_component(Bar)] fn bar() -> Html { html!() }
//! #[function_component(MyContent)]
//! pub fn my_content() -> Html {
//!   html!(
//!     <Switch<AppRoute> render={|target|match target {
//!       AppRoute::Index => html!(<Index/>),
//!       AppRoute::Foo => html!(<Foo/>),
//!       AppRoute::Bar => html!(<Bar/>),
//!     }}/>
//!   )
//! }
//! ```
//!
//! The `Switch` component does not have any children, as its content is evaluated from the `render`
//! callback.
//!
//! If no target matched, then none of the switches will match either. If is possible to define a
//! default target on the router.
//!
//! ### Nesting
//!
//! When nesting, first the structure must be declared. Let's adapt the example from above:
//!
//! ```
//! # use yew_nested_router_macros::Target;
//! #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! pub enum AppRoute {
//!   #[target(index)]
//!   Index,
//!   Foo(#[target(default)] Details),
//!   Bar {
//!     id: String,
//!     #[target(nested, default)]
//!     details: Details,
//!   },
//! }
//!
//! #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! pub enum Details {
//!   Overview,
//!   Code,
//!   Metrics,
//! }
//!
//! impl Default for Details {
//!   fn default() -> Self {
//!     Self::Overview
//!   }
//! }
//! ```
//!
//! This changed the following:
//!
//! * It added a nested layer to `Foo`, which by default will use the `Details::Overview` target.
//! * It added a nested layer to `Bar`
//!   * Capturing a path variable into `id`
//!   * Then using a nested layer to `Details`, again using the default target.
//!
//!  This will process the following routes
//!
//! | Path | Target |
//! | ---- | ------ |
//! | `/` | `AppRoute::Index` |
//! | `/foo`, `/foo/overview` | `AppRoute::Foo({id}::Overview)` |
//! | `/foo/code` | `AppRoute::Foo(Details::Code)` |
//! | `/foo/metrics` | `AppRoute::Foo({id}::Metrics)` |
//! | `/bar` | no match |
//! | `/bar/{id}`, `/foo/{id}/overview` | `AppRoute::Bar {id: "id", details: Details::Overview}` |
//! | `/foo/{id}/code` | `AppRoute::Bar {id: "id", details: Details::Code}` |
//! | `/foo/{id}/metrics` | `AppRoute::Bar {id: "id", details: Details::Metrics}` |
//!
//! ### Scoping/Translating
//!
//! The main router will only insert an routing context for the `AppRoutes` context. Now we need to
//! translate down the next level. This is done using the [`Scope`] component, which translates
//! "from" -> "to", or `P`arent -> `C`hild.
//!
//! ```
//! # use yew::prelude::*;
//! # use yew_nested_router::prelude::*;
//! # #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! # pub enum AppRoute {
//! #   #[target(index)]
//! #   Index,
//! #   Foo(Details),
//! #   Bar,
//! # }
//! # #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! # pub enum Details {
//! #   Overview,
//! #   Code,
//! #   Metrics,
//! # }
//! #[function_component(Foo)]
//! pub fn foo() -> Html {
//!   html! (
//!     <Scope<AppRoute, Details> mapper={AppRoute::mapper_foo}>
//!       <Switch<Details> render={|target|html!(/* ... */)}/>
//!     </Scope<AppRoute, Details>>
//!   )
//! }
//! ```
//!
//! The `AppRoute::mapper_foo` function was automatically created by the `Target` derive. It
//! translates upwards and downwards between the two levels.
//!
//! **NOTE:** Targets having additional information do not get a mapper automatically created, as
//! that information might not be known on the lower levels.
//!
//! For a more complete example on nesting, see the full example in the `examples` folder.
//!
//! ### Navigating
//!
//! There is an out-of-the-box component named [`components::Link`], which allows to navigate to a
//! target. It is also possible to achieve the same, using the routing context, which can be
//! acquired using [`prelude::use_router`].
//!
//! ```
//! # use yew::prelude::*;
//! # use yew_nested_router::prelude::*;
//! # #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! # pub enum AppRoute {
//! #   #[target(index)]
//! #   Index,
//! #   Foo(Details),
//! #   Bar{id: String, #[target(nested)] details: Details},
//! # }
//! # #[derive(Clone, Debug, PartialEq, Eq, Target)]
//! # pub enum Details {
//! #   Overview,
//! #   Code,
//! #   Metrics,
//! # }
//! # #[function_component(Index)] fn index() -> Html { html!() }
//! # #[function_component(Foo)] fn foo() -> Html { html!() }
//! # #[function_component(Bar)] fn bar() -> Html { html!() }
//! use yew_nested_router::components::Link;
//!
//! #[function_component(MyContent)]
//! pub fn my_content() -> Html {
//!   html!(
//!     <>
//!       <ul>
//!         <li><Link<AppRoute> target={AppRoute::Index}>{"Index"}</Link<AppRoute>></li>
//!         <li><Link<AppRoute> target={AppRoute::Foo(Details::Overview)}>{"Foo"}</Link<AppRoute>></li>
//!         <li><Link<AppRoute> target={AppRoute::Bar{ id: "".into(), details: Details::Overview}}>{"Bar"}</Link<AppRoute>></li>
//!       </ul>
//!       <Switch<AppRoute> render={|target|match target {
//!         AppRoute::Index => html!(<Index/>),
//!         AppRoute::Foo(_) => html!(<Foo/>),
//!         AppRoute::Bar{..} => html!(<Bar/>),
//!       }}/>
//!     </>
//!   )
//! }
//! ```
//!
//! ## More examples
//!
//! See the `examples` folder.

pub mod components;
pub mod target;

mod base;
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

/// Re-export the version of gloo_history we use
pub use gloo_history as history;
/// Re-export the history implementation we use
pub use gloo_history::BrowserHistory as History;
