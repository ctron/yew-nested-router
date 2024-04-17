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
                            <li><Link<Page> to={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<B> active="active" to={B::One}>{ "One" }</Link<B>></li>
                            <li><Link<B> active="active" predicate={B::is_two} to={B::Two(View::Overview)}>{ "Two" }</Link<B>></li>
                            <li><Link<B> active="active" predicate={B::is_three} to={B::Three(View::Overview)}>{ "Three" }</Link<B>></li>
                        </ul>
                    </nav>
                </Section>
                <Switch<B> render={render_b} />
            </Scope<Page, B>>
        ),
        Page::C { value, target: _target } => html!(<Section>
            <h3>
                { format!("C ({value})") }
            </h3>
            <nav>
                <Link<Page> to={Page::B(B::Two(View::Details))}>{ "Jump to Page::B(B::Two(View::Details))" }</Link<Page>>
            </nav>
        </Section>),
        Page::D { id, target: _target } => html!(
            <Scope<Page, D> mapper={move |_| Page::mapper_d(id)}>
                <Section>
                    <h3>{ "D" }</h3>
                    <nav>
                        <ul>
                            <li><Link<Page> to={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<D> active="active" predicate={D::is_first} to={D::First}>{ "First" }</Link<D>></li>
                            <li><Link<D> active="active" predicate={D::is_second} to={D::Second}>{ "Second" }</Link<D>></li>
                        </ul>
                    </nav>
                </Section>
                <Switch<D> render={move |d| render_d(d, id)} />
            </Scope<Page, D>>
        ),
    }
}

fn render_d(d: D, id: u32) -> Html {
    match d {
        D::First => html!(<Section><h2>{format!("First; id={id}")}</h2></Section>),
        D::Second => html!(<Section><h2>{format!("Second; id={id}")}</h2></Section>),
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
                    <li><Link<View> active="active" to={View::Overview}>{ "Overview" }</Link<View>></li>
                    <li><Link<View> active="active" to={View::Details}>{ "Details" }</Link<View>></li>
                    <li><Link<View> active="active" to={View::Source}>{ "Source" }</Link<View>></li>
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
                            <li><Link<Page> active="active" to={Page::Index}>{ "Home" }</Link<Page>></li>
                            <li><Link<Page> active="active" to={Page::A}>{ "A" }</Link<Page>></li>
        
                            <li>
                                <Scope<Page, B> mapper={Page::mapper_b}>
                                    <Link<B> active="active" any=true to={B::One}>{ "B" }</Link<B>>
                                    <ul>
                                        <li><Link<Page> active="active" to={Page::B(B::One)}>{ "One" }</Link<Page>></li>
                                        <li>
                                            <Scope<B, View> mapper={B::mapper_two}>
                                                <Link<View> active="active" any=true to={View::Overview}>{ "Two" }</Link<View>>
                                                <ViewNav/>
                                            </Scope<B, View>>
                                        </li>
            
                                        <li>
                                            <Scope<B, View> mapper={B::mapper_three}>
                                                <Link<View> active="active" predicate={View::any} to={View::Overview}>{ "Three" }</Link<View>>
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
                                    <li><Link<Page> active="active" to={Page::Index}>{ "Home" }</Link<Page>></li>
                                    <li><Link<Page> active="active" to={Page::A}>{ "A" }</Link<Page>></li>
                                    <li><Link<Page> active="active" predicate={Page::is_b} to={Page::B(B::One)}>{ "B" }</Link<Page>></li>
                                    <li><Link<Page> active="active" predicate={Page::is_c} to={Page::C{value: "foo".into(), target: C::Foo{value: "value".to_string()}}}>{ "C (foo)" }</Link<Page>></li>
                                    <li><Link<Page> active="active" predicate={Page::is_c} to={Page::D {id: 0, target: D::First}}>{ "D (id=0)" }</Link<Page>></li>
                                </ul>
                            </nav>
                        </div>
                    
                        <Switch<Page> {render} default={html!(<>{"Not found"}</>)}/>
        
                    </main>
                </div>

            </div>

            <section>
                <h3>{"Extras"}</h3>
                <ul>
                    <li>
                        <Link<Page>
                            to={Page::B(B::Two(View::Details))}
                            state="Hello World!"
                        >
                            {"Push to B -> Two -> Details with state"}
                        </Link<Page>>
                    </li>
                </ul>
            </section>

            <footer>
                <Debug/>
            </footer>

        </Router<Page>>
        
    </>)
}
