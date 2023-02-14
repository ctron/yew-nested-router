use yew_nested_router::Target;

/// test with a nested struct like variant having a default.
#[test]
fn test_names() {
    #[derive(Target, Debug, Clone, PartialEq, Eq)]
    pub enum Pages {
        // having a property named "path" should work
        Details { path: String },
    }
}
