use crate::components::Debug;
use crate::targets::*;
use yew::prelude::*;
use yew_nested_router::components::*;
use yew_nested_router::prelude::*;

use crate::components::*;

fn render(page: Page) -> Html {
    match page {
        Page::Index => html!(<Section>
            <h3>{ "Home" }</h3>
        </Section>),
        Page::A => html!(<Section>
            <h3>{ "A" }</h3>
        </Section>),
        Page::B(_) => html!(
            <Scope<Page, B> mapper={Page::mapper_b}>
                <Section>
                    <h3>{ "B" }</h3>
                    <nav>
                        <ul>
                            <li><Link<Page> target={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<B> active="active" target={B::One}>{ "One" }</Link<B>></li>
                            <li><Link<B> active="active" predicate={B::is_two} target={B::Two(View::Overview)}>{ "Two" }</Link<B>></li>
                            <li><Link<B> active="active" predicate={B::is_three} target={B::Three(View::Overview)}>{ "Three" }</Link<B>></li>
                        </ul>
                    </nav>
                </Section>
                <Switch<B> render={render_b} />
            </Scope<Page, B>>
        ),
        Page::C { value } => html!(<Section>
            <h3>
                { format!("C ({value})") }
            </h3>
            <nav>
                <Link<Page> target={Page::B(B::Two(View::Details))}>{ "Jump to Page::B(B::Two(View::Details))" }</Link<Page>>
            </nav>
        </Section>),
    }
}

fn render_b(b: B) -> Html {
    match b {
        B::One => html!(<Section><h2>{"One"}</h2></Section>),
        B::Two(_) => html!(
            <Scope<B, View> mapper={B::mapper_two}>
                <Section>
                    <h3>{"Two"}</h3>
                    <ViewNav/>
                </Section>
                <ViewComponent/>
            </Scope<B, View>>
        ),
        B::Three(_) => html!(
            <Scope<B, View> mapper={B::mapper_three}>
                <Section>
                    <h3>{"Three"}</h3>
                    <ViewNav/>
                </Section>
                <ViewComponent/>
            </Scope<B, View>>
        ),
    }
}

#[function_component(ViewNav)]
pub fn view_nav() -> Html {
    html!(
        <>
            <nav>
                <ul>
                    <li><Link<View> active="active" target={View::Overview}>{ "Overview" }</Link<View>></li>
                    <li><Link<View> active="active" target={View::Details}>{ "Details" }</Link<View>></li>
                    <li><Link<View> active="active" target={View::Source}>{ "Source" }</Link<View>></li>
                </ul>
            </nav>
        </>
    )
}

#[function_component(ViewComponent)]
pub fn view() -> Html {
    html!(
        <>
            <Switch<View> render={|view| match view {
                View::Overview => html!(<Section><h3>{"Overview"}</h3></Section>),
                View::Details => html!(<Section><h3>{"Details"}</h3></Section>),
                View::Source => html!(<Section><h3>{"Source"}</h3></Section>),
            }}/>
        </>
    )
}

#[function_component(Application)]
pub fn app() -> Html {
    html!(<>
        
        <Router<Page>>

            <div style="display: flex;">

                <aside>
                    <nav>
                        <h2>{"Outside"}</h2>
        
                        <ul>
                            <li><Link<Page> active="active" target={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<Page> active="active" target={Page::A}>{ "A" }</Link<Page>></li>
        
                            <li>
                                <Scope<Page, B> mapper={Page::mapper_b}>
                                    <Link<B> active="active" any=true target={B::One}>{ "B" }</Link<B>>
                                    <ul>
                                        <li><Link<Page> active="active" target={Page::B(B::One)}>{ "One" }</Link<Page>></li>
                                        <li>
                                            <Scope<B, View> mapper={B::mapper_two}>
                                                <Link<View> active="active" any=true target={View::Overview}>{ "Two" }</Link<View>>
                                                <ViewNav/>
                                            </Scope<B, View>>
                                        </li>
            
                                        <li>
                                            <Scope<B, View> mapper={B::mapper_three}>
                                                <Link<View> active="active" predicate={View::any} target={View::Overview}>{ "Three" }</Link<View>>
                                                <ViewNav/>
                                            </Scope<B, View>>
                                        </li>
                                    </ul>
                                </Scope<Page, B>>
                            </li>
                        </ul>
                    </nav>
                </aside>
        
                <div>
        
                    <h2>{"Nested"}</h2>
                
                    <main>
                
                        <div>
                            <h3>{"Main"}</h3>
                    
                            <nav>
                                <ul> 
                                    <li><Link<Page> active="active" target={Page::Index}>{ "Home" }</Link<Page>></li>
                                    <li><Link<Page> active="active" target={Page::A}>{ "A" }</Link<Page>></li>
                                    <li><Link<Page> active="active" predicate={Page::is_b} target={Page::B(B::One)}>{ "B" }</Link<Page>></li>
                                    <li><Link<Page> active="active" predicate={Page::is_c} target={Page::C{value: "foo".into()}}>{ "C (foo)" }</Link<Page>></li>
                                </ul>
                            </nav>
                        </div>
                    
                        <Switch<Page> {render} default={html!(<>{"Not found"}</>)}/>
        
                    </main>
                </div>

            </div>

            <footer>
                <Debug/>
            </footer>

        </Router<Page>>
        
    </>)
}
