use crate::navigation::{NavigationContext, Request};
use crate::switch::SwitchContext;
use crate::target::Target;
use gloo_history::{AnyHistory, BrowserHistory, History, HistoryListener, Location};
use std::fmt::Debug;
use yew::prelude::*;

/// Properties for the [`Router`] component.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps {
    /// The content to render.
    pub children: Children,
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

impl<S> Component for Router<S>
where
    S: Target + 'static,
{
    type Message = Msg;
    type Properties = RouterProps;

    fn create(ctx: &Context<Self>) -> Self {
        let history = AnyHistory::Browser(BrowserHistory::new());

        let cb = ctx.link().callback(Msg::RouteChanged);

        let target = Self::parse_location(history.location());

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

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("update: {msg:?}");

        match msg {
            Msg::RouteChanged(location) => {
                let target = Self::parse_location(location);
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
            .map(|t| t.render_path())
            .unwrap_or_default();

        let navigation = NavigationContext {
            base,
            parent: ctx.link().callback(Msg::ChangeRoute),
        };

        let switch = SwitchContext {
            navigation: navigation.clone(),
            target: self.target.clone(),
        };

        html! (
            <ContextProvider<NavigationContext> context={navigation}>
                <ContextProvider<SwitchContext<S>> context={switch}>
                    { for ctx.props().children.iter() }
                </ContextProvider<SwitchContext<S>>>
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
