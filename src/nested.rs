use crate::navigation::{NavigationContext, Request};
use crate::router::RouterContext;
use crate::target::Target;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct NestedProps<T: Target> {
    pub target: T,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Nested)]
pub fn nested<T>(props: &NestedProps<T>) -> Html
where
    T: Target + 'static,
{
    let parent = use_context::<NavigationContext>()
        .expect("Must be nested under a Router or Nested component of the target type");

    let navigation = use_memo(
        |(parent, target)| {
            let parent = parent.clone();
            let local_base = target.render_self();
            let mut global_base = parent.global_base().clone();
            global_base.extend(parent.local_base.clone());

            NavigationContext {
                global_base,
                local_base,
                parent: Callback::from(move |request: Request| {
                    parent.propagate(request);
                }),
            }
        },
        (parent.clone(), props.target.clone()),
    );

    let context = use_memo(
        |(navigation, target)| RouterContext {
            navigation: navigation.clone(),
            active_target: Some(target.clone()),
        },
        (navigation.clone(), props.target.clone()),
    );

    html!(
        <ContextProvider<RouterContext<T>> context={(*context).clone()}>
            <ContextProvider<NavigationContext> context={(*navigation).clone()}>
                { for props.children.iter() }
            </ContextProvider<NavigationContext>>
        </ContextProvider<RouterContext<T>>>
    )
}
