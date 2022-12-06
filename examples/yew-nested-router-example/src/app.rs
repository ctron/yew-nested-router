use crate::components::Debug;
use crate::targets::*;
use yew::prelude::*;
use yew_nested_router::components::*;
use yew_nested_router::prelude::*;

use crate::components::*;

fn render(page: Page) -> Html {
    match page {
        Page::Index => html!(<Section>
            <h2>{ "Home" }</h2>
        </Section>),
        Page::A => html!(<Section>
            <h2>{ "A" }</h2>
        </Section>),
        Page::B(target) => html!(
            <Nested<B> {target}>
                <Section>
                    <h2>{ "B" }</h2>
                    <nav>
                        <ul>
                            <li><Link<Page> target={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<B> active="active" target={B::One}>{ "One" }</Link<B>></li>
                            <li><Link<B> active="active" target={B::Two(View::Overview)}>{ "Two" }</Link<B>></li>
                            <li><Link<B> active="active" target={B::Three(View::Overview)}>{ "Three" }</Link<B>></li>
                        </ul>
                    </nav>
                </Section>
                <Switch<B> render={render_b} />
            </Nested<B>>
        ),
        Page::C { value } => html!(<Section>
            <h2>
                { format!("C ({value})") }
            </h2>
            <nav>
                <Link<Page> target={Page::B(B::Two(View::Details))}>{ "Jump to Page::B(B::Two(View::Details))" }</Link<Page>>
            </nav>
        </Section>),
    }
}

fn render_b(b: B) -> Html {
    match b {
        B::One => html!(<Section><h2>{"One"}</h2></Section>),
        B::Two(target) => html!(
            <Nested<View> {target}>
                <Section>
                    <h2>{"Two"}</h2>
                    <ViewNav/>
                </Section>
                <ViewComponent/>
            </Nested<View>>
        ),
        B::Three(target) => html!(
            <Nested<View> {target}>
                <Section>
                    <h2>{"Three"}</h2>
                    <ViewNav/>
                </Section>
                <ViewComponent/>
            </Nested<View>>
        ),
    }
}

fn render_view(view: View) -> Html {
    match view {
        View::Overview => html!(<Section><h2>{"Overview"}</h2></Section>),
        View::Details => html!(<Section><h2>{"Details"}</h2></Section>),
        View::Source => html!(<Section><h2>{"Source"}</h2></Section>),
    }
}

#[function_component(ViewNav)]
pub fn view_nav() -> Html {
    html!(
        <>
            <nav>
                <li><Link<View> active="active" target={View::Overview}>{ "Overview" }</Link<View>></li>
                <li><Link<View> active="active" target={View::Details}>{ "Details" }</Link<View>></li>
                <li><Link<View> active="active" target={View::Source}>{ "Source" }</Link<View>></li>
            </nav>
        </>
    )
}

#[function_component(ViewComponent)]
pub fn view() -> Html {
    html!(
        <>
            <Switch<View> render={render_view}/>
        </>
    )
}

#[function_component(Application)]
pub fn app() -> Html {
    html!(<>
        
        <Router<Page>>

            <main>
        
                <div>
                    <h2>{"Main"}</h2>
            
                    <nav>
                        <ul> 
                            <li><Link<Page> active="active" target={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<Page> active="active" target={Page::A}>{ "A" }</Link<Page>></li>
                            <li><Link<Page> active="active" target={Page::B(B::One)}>{ "B" }</Link<Page>></li>
                            <li><Link<Page> active="active" target={Page::C{value: "foo".into()}}>{ "C (foo)" }</Link<Page>></li>
                        </ul>
                    </nav>
                </div>
            
                <Switch<Page> {render} default={html!(<>{"Not found"}</>)}/>

            </main>
        
        <footer>
            <Debug/>
        </footer>
        
        </Router<Page>>
        
    </>)
}
