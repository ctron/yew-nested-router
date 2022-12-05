use crate::prelude::use_router;
use crate::target::Target;
use yew::prelude::*;

/// Properties for the [`Link`] component.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LinkProps<T>
where
    T: Target,
{
    /// It's children, rendered inside the element.
    pub children: Children,

    /// The link target.
    pub target: T,

    /// The element to render, default to `<a>`.
    #[prop_or_else(default::element)]
    pub element: String,

    /// CSS classes which are always present.
    #[prop_or_default]
    pub class: Classes,

    /// CSS classes which are added when the target is the active route.
    #[prop_or_default]
    pub active: Classes,

    /// CSS classes which are added when the target is not the active route.
    #[prop_or_default]
    pub inactive: Classes,
}

mod default {
    pub fn element() -> String {
        "a".to_string()
    }
}

/// A link component, navigating to a [`Target`] on the `onclick` event.
#[function_component(Link)]
pub fn link<T>(props: &LinkProps<T>) -> Html
where
    T: Target + 'static,
{
    let router = use_router().expect("Need Router or Nested component");

    let mut classes = props.class.clone();

    match router.is_active(&props.target) {
        true => classes.extend(props.active.clone()),
        false => classes.extend(props.inactive.clone()),
    }

    let target = props.target.clone();
    let onclick = Callback::from(move |_| router.go(target.clone()));

    html!(
        <@{props.element.clone()}
            class={classes}
            {onclick}
        >
            { for props.children.iter() }
        </@>
    )
}
