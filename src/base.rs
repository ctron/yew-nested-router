use wasm_bindgen::JsCast;
use web_sys::Url;

fn extract_base(base: String) -> Option<String> {
    let url = Url::new(&base).ok();
    url.map(|u| u.pathname().trim_end_matches('/').to_string())
}

pub(crate) fn eval_base() -> Option<String> {
    if let Ok(Some(base)) = gloo_utils::document().query_selector("base[href]") {
        let base = extract_base(base.unchecked_into::<web_sys::HtmlBaseElement>().href());

        // log::info!("Base: {base:?}");
        return base;
    }

    None
}
