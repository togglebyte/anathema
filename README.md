# Anathema

[![Crates.io](https://img.shields.io/crates/v/anathema.svg)](https://crates.io/crates/anathema)
[![Docs.rs](https://img.shields.io/docsrs/anathema)](https://docs.rs/anathema)

Anathema is a library for building text user interfaces using composable components, with a custom markup language.

It was created with the intent to give developers a fast and easy way to build text user interfaces
(TUIs) and ship the template(s) alongside the application, giving the end user the option to customise the application to their liking.

By separating the layout from the rest of the application, 
reducing the amount of code needed to express your design,
and featuring hot reloading it becomes incredibly fast to iterate over the design.

Anathema has a single pass layout inspired by Flutter and Swift UI.

## Supported features

* Hot reloading
* Reactive templates
* Template functions
* Third party components
* Distributable templates

**Note** Anathema should be considered beta for now.

See [The Guide](https://togglebyte.github.io/anathema-guide/) for getting started.

```
hstack [width: 40, height: 10]
    // Left pane
    expand [factor: 1]
        border
            vstack
                for item in [1, 2, 3]
                    text "Item " item

    // Right pane
    expand [factor: 4]
        border
            expand
                text "This isn't where I parked my car!"
```
output
```
┌──────┐┌──────────────────────────────┐
│Item 1││This isn't where I parked my  │
│Item 2││car!                          │
│Item 3││                              │
└──────┘│                              │
        │                              │
        │                              │
        │                              │
        │                              │
        └──────────────────────────────┘
```
