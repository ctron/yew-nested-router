use wasm_bindgen::JsValue;
use yew::html::IntoPropValue;

/// A page state value
#[derive(PartialEq, Debug, Clone)]
pub struct State(pub(crate) JsValue);

impl State {
    pub const fn null() -> Self {
        State(JsValue::null())
    }
}

impl From<JsValue> for State {
    fn from(value: JsValue) -> Self {
        Self(value)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::null()
    }
}

impl IntoPropValue<State> for &str {
    fn into_prop_value(self) -> State {
        State(JsValue::from_str(self))
    }
}
