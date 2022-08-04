# Anathema

A TUI library with a custom template language and runtime


```
hstack [width: 40, height: 10]:
    // Left pane
    expand [factor: 1]:
        border:
            vstack:
                text: "Item 1"
                text: "Item 2"
                text: "Item 3"

    // Right pane
    expand [factor: 4]:
        border:
            expand:
                text: "This isn't where I parked my car!"
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
