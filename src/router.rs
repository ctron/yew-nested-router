use crate::base;
use crate::scope::ScopeContext;
use crate::target::Target;
use gloo_history::{AnyHistory, BrowserHistory, History, HistoryListener, Location};
use std::borrow::Cow;
use std::fmt::Debug;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct RouterContext<T>
where
    T: Target,
{
    /// The base URL of the page
    pub(crate) base: Rc<String>,
    pub(crate) scope: Rc<ScopeContext<T>>,
    // The active target
    pub active_target: Option<T>,
}

impl<T> RouterContext<T>
where
    T: Target,
{
    /// Push a new state to the history. This changes the current target, but doesn't leave the page.
    pub fn push(&self, target: T) {
        self.scope.push(target);
    }

    /// Render the path of target.
    ///
    /// This includes the parenting scopes as well as the "base" URL of the document.
    pub fn render_target(&self, target: T) -> String {
        self.scope.collect(target)
    }

    /// Check if the provided target is the active target
    pub fn is_same(&self, target: &T) -> bool {
        match &self.active_target {
            Some(current) => current == target,
            None => false,
        }
    }

    /// Check if the target is active.
    ///
    /// This is intended for components to find out if their target, or part of their target
    /// is active. If the function is provided with a predicate, then this will override the
    /// decision process. Otherwise function will check if the provided `target` is the same as
    /// the active target.
    ///
    /// Assume you have a nested navigation tree. The active state of a leaf entry would be
    /// identified by the target being "the same". While branch entries would need to provide a
    /// predicate, as there is no "value" to compare to.
    ///
    /// A component supporting this model can provide two properties: a target, and an optional
    /// predicate. The user can then configure this accordingly. The component can simply pass
    /// the information to this function to perform the check.
    pub fn is_active(&self, target: &T, predicate: Option<&Callback<T, bool>>) -> bool {
        match predicate {
            Some(predicate) => self
                .active_target
                .clone()
                .map(|target| predicate.emit(target))
                .unwrap_or_default(),
            None => self.is_same(target),
        }
    }

    /// Get the active target, this may be [`None`], in the case this branch doesn't have an
    /// active target.
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

    /// The default target to use in case none matched.
    #[prop_or_default]
    pub default: Option<T>,

    /// The application base.
    ///
    /// Defaults to an empty string or the content of the `href` attribute of the `<base>` element.
    ///
    /// This can be used in case the application is hosted on a sub path to adapt paths generated
    /// and expected by the router.
    ///
    /// ## Usage with `trunk`
    ///
    /// If you are using `trunk` to build the application, you can add the following to your
    /// `index.html` file:
    ///
    /// ```html
    /// <head>
    ///   <base data-trunk-public-url/>
    /// </head>
    /// ```
    ///
    /// This will automatically populate the `<base>` element with the root provided using the
    /// `--public-url` argument.
    #[prop_or_default]
    pub base: Option<String>,
}

#[derive(Debug)]
#[doc(hidden)]
pub enum Msg<T: Target> {
    RouteChanged(Location),
    ChangeTarget(T),
}

/// Top-level router component.
pub struct Router<T: Target> {
    history: AnyHistory,
    _listener: HistoryListener,
    target: Option<T>,

    scope: Rc<ScopeContext<T>>,
    router: RouterContext<T>,

    base: Rc<String>,
}

impl<T> Component for Router<T>
where
    T: Target + 'static,
{
    type Message = Msg<T>;
    type Properties = RouterProps<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let history = AnyHistory::Browser(BrowserHistory::new());

        let cb = ctx.link().callback(Msg::RouteChanged);

        let base = Rc::new(
            ctx.props()
                .base
                .clone()
                .or_else(base::eval_base)
                .unwrap_or_default(),
        );

        let target =
            Self::parse_location(&base, history.location()).or_else(|| ctx.props().default.clone());

        let listener = {
            let history = history.clone();
            history.clone().listen(move || {
                cb.emit(history.location());
            })
        };

        let (scope, router) = Self::build_context(base.clone(), &target, ctx);

        Self {
            history,
            _listener: listener,
            target,
            scope,
            router,
            base,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // log::debug!("update: {msg:?}");

        match msg {
            Msg::RouteChanged(location) => {
                let target = Self::parse_location(&self.base, location)
                    .or_else(|| ctx.props().default.clone());
                if target != self.target {
                    self.target = target;
                    self.sync_context(ctx);
                    return true;
                }
            }
            Msg::ChangeTarget(target) => {
                // log::debug!("Pushing state: {:?}", request.path);
                let route = Self::render_target(&self.base, &target);
                // log::debug!("Push URL: {route}");
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
        let scope = self.scope.clone();
        let router = self.router.clone();

        html! (
            <ContextProvider<ScopeContext<T>> context={(*scope).clone()}>
                <ContextProvider<RouterContext<T >> context={router}>
                    { for ctx.props().children.iter() }
                </ContextProvider<RouterContext<T >>>
            </ContextProvider<ScopeContext<T>>>
        )
    }
}

impl<T: Target> Router<T> {
    fn render_target(base: &str, target: &T) -> String {
        format!(
            "{}/{}",
            base,
            target
                .render_path()
                .into_iter()
                .map(|segment| urlencoding::encode(&segment).to_string())
                .collect::<Vec<_>>()
                .join("/")
        )
    }

    fn parse_location(base: &str, location: Location) -> Option<T> {
        // get the current path
        let path = location.path();
        // if the prefix doesn't match, nothing will
        if !path.starts_with(&base) {
            return None;
        }
        // split off the prefix
        let (_, path) = path.split_at(base.len());
        // log::debug!("Path: {path}");

        // parse into path segments
        let path: Result<Vec<Cow<str>>, _> = path
            .split('/')
            .skip(1)
            // urldecode in the process
            .map(|segment| urlencoding::decode(segment))
            .collect();

        // get a path, or return none if we had an urldecode error
        let path = match &path {
            Ok(path) => path.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
            Err(_) => return None,
        };

        // parse the path into a target
        // log::debug!("Path: {path:?}");
        let target = T::parse_path(&path);
        // log::debug!("New target: {target:?}");

        // done
        target
    }

    fn sync_context(&mut self, ctx: &Context<Self>) {
        let (scope, router) = Self::build_context(self.base.clone(), &self.target, ctx);
        self.scope = scope;
        self.router = router;
    }

    fn build_context(
        base: Rc<String>,
        target: &Option<T>,
        ctx: &Context<Self>,
    ) -> (Rc<ScopeContext<T>>, RouterContext<T>) {
        let scope = Rc::new(ScopeContext {
            upwards: ctx.link().callback(Msg::ChangeTarget),
            collect: {
                let base = base.clone();
                Callback::from(move |target| Self::render_target(&base, &target))
            },
        });

        let router = RouterContext {
            base,
            scope: scope.clone(),
            active_target: target.clone(),
        };

        (scope, router)
    }
}

#[hook]
/// Get access to the router.
///
/// The hook requires to be called from a component which is nested into a [`Router`] component of
/// the type `T` provided here. If not, it will return [`None`].
pub fn use_router<T>() -> Option<RouterContext<T>>
where
    T: Target + 'static,
{
    use_context()
}
