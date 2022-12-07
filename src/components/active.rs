use crate::prelude::Target;
use crate::router::use_router;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ActiveProps<T>
where
    T: Target,
{
    #[prop_or_default]
    pub target: Option<T>,

    pub children: Children,

    #[prop_or_else(default::element)]
    pub element: String,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub active: Classes,

    #[prop_or_default]
    pub inactive: Classes,
}

mod default {
    pub fn element() -> String {
        "span".to_string()
    }
}

/// A style element, allowing to add cass classes based on the target's active state.
#[function_component(Active)]
pub fn active<T>(props: &ActiveProps<T>) -> Html
where
    T: Target,
{
    let router = use_router().expect("Need Router or Nested component");

    let mut class = props.class.clone();

    let active = match &props.target {
        Some(target) => router.is_same(target),
        None => router.active().is_some(),
    };

    match active {
        true => class.extend(props.active.clone()),
        false => class.extend(props.inactive.clone()),
    }

    html!(
        <@{props.element.clone()} {class}>
            { for props.children.iter() }
        </@>
    )
}
