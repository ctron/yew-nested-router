use std::fmt::Debug;

pub trait Target: Clone + Debug + PartialEq + 'static {
    fn render_path(&self) -> Vec<String>;

    fn parse_path(path: &[&str]) -> Option<Self>;
}
