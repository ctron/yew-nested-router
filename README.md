# Yew Nested Router

[![crates.io](https://img.shields.io/crates/v/yew-nested-router.svg)](https://crates.io/crates/yew-nested-router)
[![docs.rs](https://docs.rs/yew-nested-router/badge.svg)](https://docs.rs/yew-nested-router)

A router for Yew that supports nesting.

## Example

In a nutshell, you define a main router entry point (`<Router<T>>`) where you can switch the rendering (`<Switch<T>>`).
You can then translate down to the next layer (`Scope<P, C>`), so that you can switcher rendering again (`Switch<C>`).

See a complete example: [examples/yew-nested-router-example](examples/yew-nested-router-example), which
you can run using:

```shell
trunk serve examples/yew-nested-router-example/index.html
```

## Rationale

This project was born out of [the need for a nesting router](https://github.com/yewstack/yew/issues/1853) using Yew.
This used to work in past releases, but starting  with 0.19, the router implementation was swapped out for a new one
which didn't properly support nesting. It still  was possible to [backport](https://github.com/ctron/yew-router) the
old router to Yew 0.19, using the Yew Agent project.

However, in Yew Agent 0.2, which is required by Yew 0.20, the "context" agent was removed. So it was no longer possible
to backport the old router implementation.

This is a new, from scratch, implementation of a router for Yew. It supports nesting, and using Rust types for routes.

## Goals and opinions

This is not a drop-in replacement for the old Yew router. This section briefly describes which choices this
implementation made:

* The router supports a nested structured, where the child elements don't need to be aware of their parents.
  This is required in many cases, where one want to re-use lower level routes on different top level routes.
* Routes are not defined by regular expressions or other pattern matching, but by simply splitting the path of a URL
  using the forward slash (`/`) into segments.
* The implementation should try to help avoiding mistakes by using Rust's type system. But it won't fully guard against
  all mistakes which can be made. 
