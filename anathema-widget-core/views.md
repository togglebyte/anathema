Problems to solve:

* Syntax for views
* Views inside the node tree
* Views should be able to own state (casting any types)
* Register views
    * Validate names: no white spaces
* Passing state


```
fn main() {
    let mut view_table = Views::new();
    let my_view = MyView::new();
    let expressions = parse_compile_and_lol("path/to/template")?;
    view_table.register("message", my_view, expressions);

    let mut runtime = Runtime::new(view_table);
}

vstack
    for message in messages
        view "message" message
        #message message

struct MyView {
}

#[derive(Debug, State)]
struct Message {
    sender: StateValue<String>,
    text: StateValue<String>,
}

impl View for MyView {
    type State = MyState;

    fn event(&mut self, ev: Event, nodes: &mut Nodes) {
    }
}
```

```
nodes
    .query()
    .by_attrib("foreground", Color::Red)
    .by_tag("text")
    .for_each(|node| {
        
    })

for message in chat_messages
    view "chat-message" message

for message in chat_messages
    hstack
        text "username: "
            span [foreground: red] "floppy"
        text chat.message
```
