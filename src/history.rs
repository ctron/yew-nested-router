use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::JsValue;

thread_local! {
    static INSTANCE: RefCell<InnerHistory> = RefCell::new(InnerHistory {
        listeners: vec![],
    });
}

/// Handle to a history listener.
///
/// Disposes the listener when dropped.
pub struct HistoryListener(Rc<CallbackFn>);

pub struct History;

impl History {
    /// Subscribe to events of the browser history.
    ///
    /// This will receive events when popping items from the stack, as well as changes triggered by calling
    /// [`History::push_state`].
    #[must_use = "The listener will only be active for as long as the returned instance exists."]
    pub fn listener<F: Fn() + 'static>(f: F) -> HistoryListener {
        INSTANCE.with(|instance| instance.borrow_mut().listener(f))
    }

    /// Push a new state to the history stack.
    ///
    /// This will send out events and update the browser's history. Ultimately calling
    /// [`web_sys::History::push_state_with_url`].
    pub fn push_state(state: JsValue, url: &str) -> Result<(), JsValue> {
        INSTANCE.with(|instance| instance.borrow_mut().push_state(state, url))
    }
}

type CallbackFn = dyn Fn() + 'static;

struct InnerHistory {
    listeners: Vec<Weak<CallbackFn>>,
}

impl InnerHistory {
    fn push_state(&mut self, state: JsValue, url: &str) -> Result<(), JsValue> {
        let result = gloo_utils::history().push_state_with_url(&state, "", Some(url));
        self.notify();
        result
    }

    fn listener<F: Fn() + 'static>(&mut self, f: F) -> HistoryListener {
        let callback = Rc::new(f) as Rc<CallbackFn>;
        self.listeners.push(Rc::downgrade(&callback));
        HistoryListener(callback)
    }

    fn notify(&mut self) {
        let mut new = vec![];

        for listener in &mut self.listeners {
            if let Some(cb) = listener.upgrade() {
                (*cb)();
                new.push(listener.clone());
            }
        }

        self.listeners = new;
    }
}
