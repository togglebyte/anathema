Start:
* Constraint (screen size)
* Expressions
* Layout

Expressions -> Nodes

Nodes -> next

Next will either build a node or get one from the cache.

Builder:

Build node -> 
    | Single node -> layout
    | Loop for each -> layout

Single node -> WidgetContainer -> Layout



Can we do this:

WidgetContainer -> layout -> new Builder(constraints, layout)

This means the builder has to be able to return a size

* Builder needs to know when to stop building
* How to add sizes

```
a
    b
    c
        b
        b
    b


b -> single size
c -> b + b
```

Expression a -> Nodes a
Nodes a -> next(builder) -> build_single
build_single -> layout
