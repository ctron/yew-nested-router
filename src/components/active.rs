use crate::prelude::Target;
use crate::router::use_router;
use yew::prelude::*;

/// Properties for [`Active`]
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ActiveProperties<T>
where
    T: Target,
{
    /// The target to check for
    #[prop_or_default]
    pub route: Option<T>,

    /// Its content
    #[prop_or_default]
    pub children: Html,

    #[prop_or_default]
    pub id: Option<AttrValue>,

    /// The HTML element to use
    #[prop_or_else(default::element)]
    pub element: String,

    /// base classes
    #[prop_or_default]
    pub class: Classes,

    /// additional classes when active
    #[prop_or_default]
    pub active: Classes,

    /// additional classes when inactive
    #[prop_or_default]
    pub inactive: Classes,
}

mod default {
    pub fn element() -> String {
        "span".to_string()
    }
}

/// A style element, allowing to add cass classes based on the target's active state.
#[component(Active)]
pub fn active<T>(props: &ActiveProperties<T>) -> Html
where
    T: Target,
{
    let router = use_router().expect("Need Router or Nested component");

    let mut class = props.class.clone();

    let active = match &props.route {
        Some(route) => router.is_same(route),
        None => router.active().is_some(),
    };

    match active {
        true => class.extend(props.active.clone()),
        false => class.extend(props.inactive.clone()),
    }

    html!(
        <@{props.element.clone()}
            {class}
            id={props.id.clone()}
        >
            { props.children.clone() }
        </@>
    )
}
