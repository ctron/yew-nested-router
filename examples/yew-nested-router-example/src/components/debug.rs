use crate::targets::Page;
use yew::prelude::*;
use yew_nested_router::prelude::use_router;

#[function_component(Debug)]
pub fn debug() -> Html {
    let router = use_router::<Page>();

    let current = router.and_then(|r| r.active().clone());

    html!(
        <dl>
            <dt>{"Current Route"}</dt>
            <dd><pre><code>{ format!("{current:?}") }</code></pre></dd>
        </dl>
    )
}
