use crate::navigation::{NavigationContext, Request};
use crate::target::Target;
use gloo_history::{AnyHistory, BrowserHistory, History, HistoryListener, Location};
use std::fmt::Debug;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct RouterContext<T>
where
    T: Target,
{
    pub(crate) navigation: NavigationContext,
    // The active target
    pub active_target: Option<T>,
}

impl<T> RouterContext<T>
where
    T: Target,
{
    pub fn go(&self, target: T) {
        self.navigation.go(target);
    }

    /// check if the provided is the current target
    pub fn is_same(&self, target: &T) -> bool {
        match &self.active_target {
            Some(current) => current == target,
            None => false,
        }
    }

    pub fn is_active(&self, target: &T) -> bool {
        match &self.active_target {
            Some(current) => {
                let current = current.render_self();
                let target = target.render_self();
                target.starts_with(&current)
            }
            None => false,
        }
    }
}

/// Properties for the [`Router`] component.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<T>
where
    T: Target,
{
    /// The content to render.
    pub children: Children,

    #[prop_or_default]
    pub default: Option<T>,
}

#[derive(Debug)]
#[doc(hidden)]
pub enum Msg {
    RouteChanged(Location),
    ChangeRoute(Request),
}

/// Top-level router component.
pub struct Router<T: Target> {
    history: AnyHistory,
    _listener: HistoryListener,
    target: Option<T>,
}

impl<T> Component for Router<T>
where
    T: Target + 'static,
{
    type Message = Msg;
    type Properties = RouterProps<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let history = AnyHistory::Browser(BrowserHistory::new());

        let cb = ctx.link().callback(Msg::RouteChanged);

        let target =
            Self::parse_location(history.location()).or_else(|| ctx.props().default.clone());

        let listener = {
            let history = history.clone();
            history.clone().listen(move || {
                cb.emit(history.location());
            })
        };

        Self {
            history,
            _listener: listener,
            target,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("update: {msg:?}");

        match msg {
            Msg::RouteChanged(location) => {
                let target = Self::parse_location(location).or_else(|| ctx.props().default.clone());
                if target != self.target {
                    self.target = target;
                    return true;
                }
            }
            Msg::ChangeRoute(request) => {
                log::debug!("Pushing state: {:?}", request.path);
                let route = format!("/{}", request.path.join("/"));
                self.history.push(route);
            }
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let base = self
            .target
            .as_ref()
            .map(|t| t.render_self())
            .unwrap_or_default();

        let navigation = NavigationContext {
            base,
            parent: ctx.link().callback(Msg::ChangeRoute),
        };

        let router = RouterContext {
            navigation: navigation.clone(),
            active_target: self.target.clone(),
        };

        html! (
            <ContextProvider<NavigationContext> context={navigation}>
                <ContextProvider<RouterContext<T >> context={router}>
                    { for ctx.props().children.iter() }
                </ContextProvider<RouterContext<T >>>
            </ContextProvider<NavigationContext>>
        )
    }
}

impl<T: Target> Router<T> {
    fn parse_location(location: Location) -> Option<T> {
        let path: Vec<&str> = location.path().split('/').skip(1).collect();
        log::debug!("Path: {path:?}");
        let target = T::parse_path(&path);
        log::debug!("New target: {target:?}");
        target
    }
}

#[hook]
pub fn use_router<T>() -> Option<RouterContext<T>>
where
    T: Target + 'static,
{
    use_context::<RouterContext<T>>()
}
