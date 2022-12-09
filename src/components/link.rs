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

    #[prop_or_default]
    pub any: bool,

    #[prop_or_default]
    pub predicate: Option<Callback<T, bool>>,

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
    let router = use_router::<T>().expect("Need Router or Nested component");

    let mut classes = props.class.clone();

    let active = match props.any {
        true => {
            let active = router.active();
            active.is_some()
        }
        false => match &props.predicate {
            Some(predicate) => router
                .active()
                .clone()
                .map(|t| predicate.emit(t))
                .unwrap_or(false),
            None => router.is_same(&props.target),
        },
    };

    match active {
        true => classes.extend(props.active.clone()),
        false => classes.extend(props.inactive.clone()),
    }

    let target = props.target.clone();
    let onclick = Callback::from(move |_| router.push(target.clone()));

    html!(
        <@{props.element.clone()}
            class={classes}
            {onclick}
        >
            { for props.children.iter() }
        </@>
    )
}
