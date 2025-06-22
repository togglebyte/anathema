# Anathema

[![Crates.io](https://img.shields.io/crates/v/anathema.svg)](https://crates.io/crates/anathema)
[![Docs.rs](https://img.shields.io/docsrs/anathema)](https://docs.rs/anathema)

A TUI library with a custom template language and runtime.

**Note** Anathema should be considered beta for now.

[Getting started](https://togglebyte.github.io/anathema-guide/)

```yml
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
