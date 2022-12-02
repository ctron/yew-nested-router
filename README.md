# Yew Nested Router

A router for Yew which supports nesting.

## State

This is an early state, born out of the necessity to have a replacement for the old Yew router (which) supported nesting, but relies on Yew actors, which is mostly gone in Yew 0.20.

Compared to [ctron/yew-router](https://github.com/ctron/yew-router) (which is a backport of the old Yew router to a newer Yew version), this project is a complete rewrite. So it will not be a drop-in replacement, and might behave differently.

The following things have to be done before a release makes sense:

* [ ] Allow using path variables (like `/application/{applications}/device/{device}/settings`).
* [ ] Simplify the enum definition using either a `derive`, or macro.
* [ ] Think about handling state, currently this only handles the path. The History API however can do more.
* [ ] Probably some more, feel free to raise issues.

## Example

Also see a complete example: [examples/yew-nested-router-example](examples/yew-nested-router-example), which
you can run using:

```shell
trunk serve examples/yew-nested-router-example/index.html
```
