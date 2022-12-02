use crate::prelude::Target;
use crate::switch::use_switch;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ActiveProps<T: Target> {
    pub target: T,

    pub children: Children,

    #[prop_or_else(default::element)]
    pub element: String,

    #[prop_or_else(default::class)]
    pub class: Classes,

    #[prop_or_default]
    pub inactive_class: Classes,
}

mod default {
    use yew::Classes;

    pub fn element() -> String {
        "span".to_string()
    }

    pub fn class() -> Classes {
        Classes::from("active")
    }
}

#[function_component(Active)]
pub fn active<T>(props: &ActiveProps<T>) -> Html
where
    T: Target,
{
    let switch = use_switch();
    let active = switch
        .map(|s| s.is_active(&props.target))
        .unwrap_or_default();

    let class = match active {
        true => &props.class,
        false => &props.inactive_class,
    }
    .clone();

    html!(
        <@{props.element.clone()} {class}>
            { for props.children.iter() }
        </@>
    )
}
