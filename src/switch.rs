use crate::navigation::NavigationContext;
use crate::target::Target;
use std::fmt::Debug;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SwitchContext<T>
where
    T: Target,
{
    pub(crate) navigation: NavigationContext,
    pub target: Option<T>,
}

impl<T> SwitchContext<T>
where
    T: Target,
{
    pub fn go(&self, target: T) {
        self.navigation.go(target);
    }

    /// check if the provided is the current target
    pub fn is_same(&self, target: &T) -> bool {
        match &self.target {
            Some(current) => current == target,
            None => false,
        }
    }

    pub fn is_active(&self, target: &T) -> bool {
        match &self.target {
            Some(current) => {
                let current = current.render_path();
                let target = target.render_path();
                target.starts_with(&current)
            }
            None => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SwitchProps<S>
where
    S: PartialEq,
{
    pub switch: Callback<S, Html>,
    #[prop_or_default]
    pub default: Html,
}

#[function_component(Switch)]
pub fn switch<T>(props: &SwitchProps<T>) -> Html
where
    T: Target + 'static,
{
    let s = use_context::<SwitchContext<T>>().expect("Need SwitchContext");

    html!(<>
        {
            match s.target {
                Some(target) => props.switch.emit(target),
                None => props.default.clone(),
            }
        }
    </>)
}

#[hook]
pub fn use_switch<S>() -> Option<SwitchContext<S>>
where
    S: Target + 'static,
{
    use_context::<SwitchContext<S>>()
}
