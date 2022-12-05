use crate::navigation::{NavigationContext, Request};
use crate::router::RouterContext;
use crate::target::Target;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct NestedProps<T: Target> {
    pub target: T,
    pub children: Children,
}

#[function_component(Nested)]
pub fn nested<T>(props: &NestedProps<T>) -> Html
where
    T: Target + 'static,
{
    let parent = use_context::<NavigationContext>()
        .expect("Must be nested under a Router or Nested component of the target type");

    let navigation = NavigationContext {
        base: props.target.render_path(),
        parent: Callback::from(move |request: Request| {
            parent.propagate(request);
        }),
    };

    let context = RouterContext {
        navigation: navigation.clone(),
        active_target: Some(props.target.clone()),
    };

    html!(
        <ContextProvider<RouterContext<T>> {context}>
            <ContextProvider<NavigationContext> context={navigation}>
                { for props.children.iter() }
            </ContextProvider<NavigationContext>>
        </ContextProvider<RouterContext<T>>>
    )
}
