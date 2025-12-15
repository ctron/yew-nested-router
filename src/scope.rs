use crate::router::{RouterContext, StackOperation};
use crate::target::{Mapper, Target};
use wasm_bindgen::JsValue;
use yew::prelude::*;

#[derive(Debug)]
#[doc(hidden)]
pub struct NavigationTarget<T>
where
    T: Target,
{
    pub target: T,
    pub state: JsValue,
}

impl<T> NavigationTarget<T>
where
    T: Target,
{
    pub fn map<F, U>(self, f: F) -> NavigationTarget<U>
    where
        F: FnOnce(T) -> U,
        U: Target,
    {
        NavigationTarget {
            target: f(self.target),
            state: self.state,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScopeContext<C>
where
    C: Target,
{
    pub(crate) upwards: Callback<(NavigationTarget<C>, StackOperation)>,
    pub(crate) collect: Callback<C, String>,
}

impl<C> ScopeContext<C>
where
    C: Target,
{
    pub(crate) fn push(&self, target: C) {
        self.upwards.emit((
            (NavigationTarget {
                target,
                state: JsValue::null(),
            }),
            StackOperation::Push,
        ));
    }

    pub(crate) fn replace(&self, target: C) {
        self.upwards.emit((
            (NavigationTarget {
                target,
                state: JsValue::null(),
            }),
            StackOperation::Replace,
        ));
    }

    pub(crate) fn push_with(&self, target: C, state: JsValue) {
        self.upwards
            .emit((NavigationTarget { target, state }, StackOperation::Push))
    }
    pub(crate) fn replace_with(&self, target: C, state: JsValue) {
        self.upwards
            .emit((NavigationTarget { target, state }, StackOperation::Replace))
    }

    pub(crate) fn collect(&self, target: C) -> String {
        self.collect.emit(target)
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ScopeProps<P, C>
where
    P: Target,
    C: Target,
{
    pub mapper: Callback<(), Mapper<P, C>>,

    #[prop_or_default]
    pub children: Children,
}

/// A component, translating down to the next level.
#[component(Scope)]
pub fn scope<P, C>(props: &ScopeProps<P, C>) -> Html
where
    P: Target + 'static,
    C: Target + 'static,
{
    let router = use_context::<RouterContext<P>>()
        .expect("Must be nested under a Router or Nested component of the parent type");

    let parent = use_context::<ScopeContext<P>>()
        .expect("Must be nested under a Router or Nested component of the parent type");

    let Mapper { downwards, upwards } = props.mapper.emit(());

    let scope = use_memo((parent.clone(), upwards), |(parent, upwards)| {
        ScopeContext {
            upwards: {
                let parent = parent.upwards.clone();
                let upwards = upwards.clone();
                Callback::from(
                    move |(target, operation): (NavigationTarget<C>, StackOperation)| {
                        parent.emit((target.map(|target| upwards.emit(target)), operation));
                    },
                )
            },
            collect: {
                let parent = parent.collect.clone();
                let upwards = upwards.clone();
                Callback::from(move |child: C| parent.emit(upwards.emit(child)))
            },
        }
    });

    let base = router.base.clone();
    let active = router.active();

    let context = use_memo(
        (
            base,
            scope.clone(),
            active.clone().and_then(|p| downwards.emit(p)),
        ),
        |(base, scope, target)| RouterContext {
            base: base.clone(),
            scope: scope.clone(),
            active_target: target.clone(),
        },
    );

    html!(
        <ContextProvider<RouterContext<C>> context={(*context).clone()}>
            <ContextProvider<ScopeContext<C>> context={(*scope).clone()}>
                { for props.children.iter() }
            </ContextProvider<ScopeContext<C>>>
        </ContextProvider<RouterContext<C>>>
    )
}
