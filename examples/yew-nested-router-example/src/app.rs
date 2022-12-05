use std::fmt::Debug;
use yew::prelude::*;
use yew_nested_router::components::*;
use yew_nested_router::prelude::*;

use crate::components::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    A,
    B(B),
    C(String),
}

impl Target for Page {
    fn render_path(&self) -> Vec<String> {
        match self {
            Self::A => vec!["a".into()],
            Self::B(_) => vec!["b".into()],
            Self::C(value) => vec!["c".into(), value.clone()],
        }
    }

    fn parse_path(path: &[&str]) -> Option<Self> {
        match path {
            ["a"] => Some(Self::A),
            ["b"] => Some(Self::B(B::One)),
            ["b", rest @ ..] => B::parse_path(rest).map(Self::B),
            ["c", value] => Some(Self::C(value.to_string())),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum B {
    One,
    Two,
    Three,
}

impl Target for B {
    fn render_path(&self) -> Vec<String> {
        match self {
            Self::One => vec!["one".into()],
            Self::Two => vec!["two".into()],
            Self::Three => vec!["three".into()],
        }
    }

    fn parse_path(path: &[&str]) -> Option<Self> {
        match path {
            ["one"] => Some(B::One),
            ["two"] => Some(B::Two),
            ["three"] => Some(B::Three),
            _ => None,
        }
    }
}

fn switch(page: Page) -> Html {
    match page {
        Page::A => html!(<Section>{ "A" }</Section>),
        Page::B(target) => html!(
            <Nested<B> {target}>
                <Section>
                    { "B" }
                    <nav>
                        <ul>
                            <li><Link<Page> target={Page::A}>{ "Home (A)" }</Link<Page>></li>
                            <li><Link<B> active="active" target={B::One}>{ "One" }</Link<B>></li>
                            <li><Link<B> active="active" target={B::Two}>{ "Two" }</Link<B>></li>
                            <li><Link<B> active="active" target={B::Three}>{ "Three" }</Link<B>></li>
                        </ul>
                    </nav>
                    <Switch<B> switch={switch_b} />
                </Section>
            </Nested<B>>
        ),
        Page::C(value) => html!(<Section>
            <div>
                { format!("C ({value})") }
            </div>
            <nav>
                <Link<Page> target={Page::B(B::Two)}>{ "Jump to Page::B(B::Two)" }</Link<Page>>
            </nav>
        </Section>),
    }
}

fn switch_b(b: B) -> Html {
    match b {
        B::One => html!(<Section>{"One"}</Section>),
        B::Two => html!(<Section>{"Two"}</Section>),
        B::Three => html!(<Section>{"Three"}</Section>),
    }
}

#[function_component(Application)]
pub fn app() -> Html {
    html!(<>
        
        <Router<Page>>
        
            <header>
                <nav>
                    <ul> 
                        <li><Link<Page> active="active" target={Page::A}>{ "A" }</Link<Page>></li>
                        <li><Link<Page> active="active" target={Page::B(B::One)}>{ "B" }</Link<Page>></li>
                        <li><Link<Page> active="active" target={Page::C("foo".into())}>{ "C (foo)" }</Link<Page>></li>
                    </ul>
                </nav>
            </header>
            
            <main>
                <div>
                    <Switch<Page> {switch} default={html!(<>{"Not found"}</>)}/>
                </div>
            </main>
        
        </Router<Page>>
        
    </>)
}
