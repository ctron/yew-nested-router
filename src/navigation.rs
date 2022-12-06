use crate::target::Target;
use yew::Callback;

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct Request {
    pub path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NavigationContext {
    /// global base, without the local base
    pub(crate) global_base: Vec<String>,
    /// local base
    pub(crate) local_base: Vec<String>,

    pub parent: Callback<Request>,
}

impl NavigationContext {
    pub(crate) fn propagate(&self, mut request: Request) {
        // log::debug!("Propagate: {request:?}");

        let mut path = self.local_base.clone();
        path.extend(request.path);
        request.path = path;

        // log::debug!("Propagate(out): {request:?}");

        self.parent.emit(request);
    }

    pub(crate) fn go<T: Target>(&self, target: T) {
        // log::debug!("go: {target:?} (base: {:?})", self.base);

        let mut path = vec![];
        target.render_path_into(&mut path);

        // log::debug!("rendered: {path:?}");

        self.parent.emit(Request { path });
    }

    /// Get the global base
    pub(crate) fn global_base(&self) -> &Vec<String> {
        &self.global_base
    }

    /// Get the local base
    pub(crate) fn local_base(&self) -> &Vec<String> {
        &self.local_base
    }

    pub(crate) fn full_base(&self) -> Vec<String> {
        let mut result = Vec::<String>::new();
        result.extend(self.global_base.clone());
        result.extend(self.local_base.clone());
        result
    }
}
