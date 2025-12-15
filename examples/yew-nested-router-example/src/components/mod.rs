mod debug;

use yew::prelude::*;

pub use debug::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SectionProps {
    #[prop_or_default]
    pub children: Children,
}

#[component(Section)]
pub fn page(props: &SectionProps) -> Html {
    html!(
        <section>
            { for props.children.iter() }
        </section>
    )
}
