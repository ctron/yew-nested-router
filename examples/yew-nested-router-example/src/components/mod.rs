use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SectionProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Section)]
pub fn page(props: &SectionProps) -> Html {
    html!(
        <section>
            { for props.children.iter() }
        </section>
    )
}
