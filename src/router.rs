use crate::navigation::{NavigationContext, Request};
use crate::target::Target;
use gloo_history::{AnyHistory, BrowserHistory, History, HistoryListener, Location};
use std::fmt::Debug;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct RouterContext<T>
where
    T: Target,
{
    pub(crate) navigation: Rc<NavigationContext>,
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
        /*
        log::debug!(
            "is_active - target: {target:?}, active: {:?}",
            self.active_target
        );
        */

        match &self.active_target {
            Some(active) => {
                // let active = active.render_path();
                let base = self.navigation.full_base();
                let mut target_path = self.navigation.global_base().clone();
                target_path.extend(target.render_path());
                let result = target_path.starts_with(&base);
                log::debug!("is_active - full_base: {base:?}, active: {active:?}, target: {target_path:?}, result: {result}");
                result
            }
            None => false,
        }
    }

    pub fn active(&self) -> &Option<T> {
        &self.active_target
    }
}

/// Properties for the [`Router`] component.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<T>
where
    T: Target,
{
    /// The content to render.
    #[prop_or_default]
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

    navigation: Rc<NavigationContext>,
    router: RouterContext<T>,
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

        let (navigation, router) = Self::build_context(&target, ctx);

        Self {
            history,
            _listener: listener,
            target,
            navigation,
            router,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // log::debug!("update: {msg:?}");

        match msg {
            Msg::RouteChanged(location) => {
                let target = Self::parse_location(location).or_else(|| ctx.props().default.clone());
                if target != self.target {
                    self.target = target;
                    self.sync_context(ctx);
                    return true;
                }
            }
            Msg::ChangeRoute(request) => {
                // log::debug!("Pushing state: {:?}", request.path);
                let route = format!("/{}", request.path.join("/"));
                self.history.push(route);
            }
        }

        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.sync_context(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let navigation = self.navigation.clone();
        let router = self.router.clone();

        html! (
            <ContextProvider<NavigationContext> context={(*navigation).clone()}>
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
        // log::debug!("Path: {path:?}");
        let target = T::parse_path(&path);
        // log::debug!("New target: {target:?}");
        target
    }

    fn sync_context(&mut self, ctx: &Context<Self>) {
        let (navigation, router) = Self::build_context(&self.target, ctx);
        self.navigation = navigation;
        self.router = router;
    }

    fn build_context(
        target: &Option<T>,
        ctx: &Context<Self>,
    ) -> (Rc<NavigationContext>, RouterContext<T>) {
        let local_base = target.as_ref().map(|t| t.render_self()).unwrap_or_default();

        let navigation = Rc::new(NavigationContext {
            local_base,
            global_base: vec![],
            parent: ctx.link().callback(Msg::ChangeRoute),
        });

        let router = RouterContext {
            navigation: navigation.clone(),
            active_target: target.clone(),
        };

        (navigation, router)
    }
}

#[hook]
pub fn use_router<T>() -> Option<RouterContext<T>>
where
    T: Target + 'static,
{
    use_context::<RouterContext<T>>()
}
