use crate::target::Target;
use yew::Callback;

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct Request {
    pub path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NavigationContext {
    pub(crate) base: Vec<String>,

    pub parent: Callback<Request>,
}

impl NavigationContext {
    pub(crate) fn propagate(&self, mut request: Request) {
        log::info!("Propagate: {request:?}");

        let mut path = self.base.clone();
        path.extend(request.path);
        request.path = path;

        log::info!("Propagate(out): {request:?}");

        self.parent.emit(request);
    }

    pub(crate) fn go<T: Target>(&self, target: T) {
        log::info!("go: {target:?} (base: {:?})", self.base);

        self.parent.emit(Request {
            path: target.render_path(),
        });
    }
}
