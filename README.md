# Anathema

A TUI library with a custom template language and runtime.

**Note** Anathema should be considered alpha for now.

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
