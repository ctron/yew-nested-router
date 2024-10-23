use crate::prelude::{use_router, Target};
use crate::state::State;
use gloo_events::{EventListener, EventListenerOptions};
use log::{error, info};
use web_sys::HtmlElement;
use yew::prelude::*;

/// Properties for the [`Link`] component.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LinkProperties<T>
where
    T: Target,
{
    /// Its content, rendered inside the element.
    #[prop_or_default]
    pub children: Html,

    #[prop_or_default]
    pub id: Option<AttrValue>,

    /// The link target route.
    pub to: T,

    /// A state to push, if present
    #[prop_or_default]
    pub state: State,

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

    /// Suppress rendering the state to the "hash" of the "href".
    #[prop_or_default]
    pub suppress_hash: bool,

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
pub fn link<T>(props: &LinkProperties<T>) -> Html
where
    T: Target + 'static,
{
    info!("Render Link {:?}", props.to);
    let router = match use_router::<T>() {
        None => {
            error!("No router");
            return html! {"Need Router or Nested component"};
            //panic!("Need Router or Nested component")
        }
        Some(r) => {
            error!("Has Router");
            r
        }
    };

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
            None => router.is_same(&props.to),
        },
    };

    match active {
        true => class.extend(props.active.clone()),
        false => class.extend(props.inactive.clone()),
    }

    let href = match props.element.as_str() {
        "a" if !props.suppress_href => {
            Some(router.render_target_with(props.to.clone(), props.state.clone()))
        }
        _ => None,
    };

    let node_ref = use_node_ref();

    use_effect_with(
        (
            router,
            props.to.clone(),
            props.state.clone(),
            node_ref.clone(),
        ),
        |(router, to, state, node_ref)| {
            let mut listener = None;

            if let Some(element) = node_ref.cast::<HtmlElement>() {
                let router = router.clone();
                let to = to.clone();
                let state = state.clone();
                listener = Some(EventListener::new_with_options(
                    &element,
                    "click",
                    EventListenerOptions::enable_prevent_default(),
                    move |e| {
                        e.prevent_default();
                        router.push_with(to.clone(), state.clone());
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
            id={props.id.clone()}
        >
            { props.children.clone() }
        </@>
    )
}
