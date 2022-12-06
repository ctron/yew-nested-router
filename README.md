# Yew Nested Router

A router for Yew which supports nesting.

## State

This is an early state, born out of the necessity to have a replacement for the old Yew router (which) supported nesting, but relies on Yew actors, which is mostly gone in Yew 0.20.

Compared to [ctron/yew-router](https://github.com/ctron/yew-router) (which is a backport of the old Yew router to a newer Yew version), this project is a complete rewrite. So it will not be a drop-in replacement, and might behave differently.

The following things have to be done before a release makes sense:

* [x] Allow using path variables (like `/application/{applications}/device/{device}/settings`).
* [x] Simplify the enum definition using either a `derive`, or macro.
* [x] Do a better job at naming stuff.
* [ ] Better handling of the `/` (home, index) route.
* [ ] Think about handling state, currently this only handles the path. The History API however can do more.
* [ ] Probably some more, feel free to raise issues.

## Goals and opinions

This is not a drop-in replacement for the old Yew router. This section briefly describes which choices this implementation made:

* The router supports a nested structured, where the child elements don't need to be aware of their parents.
  This is required in many cases, where one want to re-use lower level routes on different top level routes.
* Routes are not but regular expressions or other pattern matching, but by simply splitting the path of a URL using the forward slash (`/`) into segments.
* The implementation should try to help avoiding mistakes by using Rust's type system. But it won't fully guard against all mistakes which can be made. 

## Example

Also see a complete example: [examples/yew-nested-router-example](examples/yew-nested-router-example), which
you can run using:

```shell
trunk serve examples/yew-nested-router-example/index.html
```
