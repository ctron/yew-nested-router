//! Routing target

use std::fmt::Debug;

/// A target for used by a router.
pub trait Target: Clone + Debug + Eq + 'static {
    fn render_path(&self) -> Vec<String>;

    fn parse_path(path: &[&str]) -> Option<Self>;
}
