use crate::prelude::use_switch;
use crate::target::Target;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LinkProps<S>
where
    S: PartialEq + Clone,
{
    pub children: Children,
    pub target: S,

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
        "a".to_string()
    }
}

#[function_component(Link)]
pub fn link<S>(props: &LinkProps<S>) -> Html
where
    S: Target + 'static,
{
    let switch = use_switch().expect("Need navigation context");

    let mut classes = props.class.clone();

    match switch.is_active(&props.target) {
        true => classes.extend(props.active.clone()),
        false => classes.extend(props.inactive.clone()),
    }

    let target = props.target.clone();
    let onclick = Callback::from(move |_| switch.go(target.clone()));

    html!(
        <@{props.element.clone()}
            class={classes}
            {onclick}
        >
            { for props.children.iter() }
        </@>
    )
}
