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
        log::debug!("Propagate: {request:?}");

        let mut path = self.base.clone();
        path.extend(request.path);
        request.path = path;

        log::debug!("Propagate(out): {request:?}");

        self.parent.emit(request);
    }

    pub(crate) fn go<T: Target>(&self, target: T) {
        log::debug!("go: {target:?} (base: {:?})", self.base);

        let mut path = vec![];
        target.render_path_into(&mut path);

        log::debug!("rendered: {path:?}");

        self.parent.emit(Request { path });
    }
}
