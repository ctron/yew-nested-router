use crate::targets::Page;
use yew::prelude::*;
use yew_nested_router::prelude::{use_router, Target};

#[function_component(Debug)]
pub fn debug() -> Html {
    let router = use_router::<Page>();

    let route = router.and_then(|r| r.active().clone());
    let path = route.as_ref().map(|r| r.render_path());
    let state = gloo_utils::history().state();

    html!(
        <dl>
            <dt>{"Active Route"}</dt>
            <dd><pre><code>{ format!("{route:?}") }</code></pre></dd>
            <dt>{"Active Path"}</dt>
            <dd><pre><code>{ format!("{path:?}") }</code></pre></dd>
            <dt>{"Active State"}</dt>
            <dd><pre><code>{ format!("{state:?}") }</code></pre></dd>
        </dl>
    )
}
