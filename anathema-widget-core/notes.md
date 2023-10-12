Start:
* Constraint (screen size)
* Expressions

Expressions -> Nodes

Nodes -> next

Next will either build a node or get one from the cache.

Builder:

Build node -> 
    | Single node -> layout
    | Loop for each -> layout


Single node -> WidgetContainer -> Layout


Pass size by mut ref

Layout <- ref mut size
