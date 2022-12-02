use crate::switch::SwitchContext;
use crate::target::Target;
use yew::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct Request {
    pub path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationContext {
    pub(crate) base: Vec<String>,

    pub parent: Callback<Request>,
}

impl NavigationContext {
    fn propagate(&self, mut request: Request) {
        log::info!("Propagate: {request:?}");

        let mut path = self.base.clone();
        path.extend(request.path);
        request.path = path;

        log::info!("Propagate(out): {request:?}");

        self.parent.emit(request);
    }

    pub(crate) fn go<T: Target>(&self, target: T) {
        log::info!("go: {target:?} (base: {:?})", self.base);

        self.parent.emit(Request {
            path: target.render_path(),
        });
    }
}

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

    let context = SwitchContext {
        navigation: navigation.clone(),
        target: Some(props.target.clone()),
    };

    html!(
        <ContextProvider<SwitchContext<T>> {context}>
            <ContextProvider<NavigationContext> context={navigation}>
                { for props.children.iter() }
            </ContextProvider<NavigationContext>>
        </ContextProvider<SwitchContext<T>>>
    )
}
