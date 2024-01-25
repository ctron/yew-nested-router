use serde::Serialize;
use wasm_bindgen::JsValue;
use yew::html::IntoPropValue;

/// A page state value
///
/// This is a thing wrapper around [`JsValue`], allowing for an easier interaction with the API,
/// especially in the context of `yew`.
#[derive(PartialEq, Debug, Clone)]
pub struct State(pub(crate) JsValue);

impl State {
    /// A `null` value
    pub const fn null() -> Self {
        State(JsValue::null())
    }

    /// Serialize a value into [`JsValue`].
    pub fn json<S: Serialize>(value: &S) -> Result<Self, serde_json::Error> {
        use gloo_utils::format::JsValueSerdeExt;
        Ok(State(JsValue::from_serde(value)?))
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

impl IntoPropValue<State> for String {
    fn into_prop_value(self) -> State {
        State(JsValue::from_str(&self))
    }
}

impl IntoPropValue<State> for &String {
    fn into_prop_value(self) -> State {
        State(JsValue::from_str(self))
    }
}

impl IntoPropValue<State> for JsValue {
    fn into_prop_value(self) -> State {
        State(self)
    }
}
