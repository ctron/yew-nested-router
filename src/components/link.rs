use crate::prelude::{use_router, Target};
use gloo_events::{EventListener, EventListenerOptions};
use web_sys::HtmlElement;
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

    /// Suppress rendering the "href" attribute in any case.
    #[prop_or_default]
    pub suppress_href: bool,

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

    let mut class = props.class.clone();

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
        true => class.extend(props.active.clone()),
        false => class.extend(props.inactive.clone()),
    }

    let href = match props.element.as_str() {
        "a" if !props.suppress_href => Some(router.render_target(props.target.clone())),
        _ => None,
    };

    let node_ref = use_node_ref();

    use_effect_with(
        (router, props.target.clone(), node_ref.clone()),
        |(router, target, node_ref)| {
            let mut listener = None;

            if let Some(element) = node_ref.cast::<HtmlElement>() {
                let router = router.clone();
                let target = target.clone();
                listener = Some(EventListener::new_with_options(
                    &element,
                    "click",
                    EventListenerOptions::enable_prevent_default(),
                    move |e| {
                        e.prevent_default();
                        router.push(target.clone());
                    },
                ));
            }

            move || drop(listener)
        },
    );

    html!(
        <@{props.element.clone()}
            {class}
            {href}
            ref={node_ref}
        >
            { for props.children.iter() }
        </@>
    )
}
