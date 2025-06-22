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
* Message passing supports async
* Message passing between components via an emitter

**Note** Anathema should be considered beta for now.

See [The Guide](https://togglebyte.github.io/anathema-guide/) for getting started.

### Example component

```rust
struct MyComponent;

impl Component for MyComponent {
    type State = MyState;
    type Message = ();

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
        dt: Duration,
    ) {
        *state.value.to_mut() += 1;
    }
}

#[derive(State)]
struct MyState {
    counter: Value<u32>,
}
```

### Example template

```
vstack
    text "the counter is " state.counter
    text "the counter will be " state.counter + 1
    if state.count > 10
        text "the counter is more than ten"
```
output
```
the counter is 11
the counter will be 12
the counter is more than ten
```

### Screenshots etc.

#### [Bubbles by Jyn](https://asciinema.org/a/LsUSqMlSu3OlhraQOSXMpCXhZ) ASCII
  cinema

#### Markdown reader by Doddi
![Markdown reader by Doddi](https://github.com/togglebyte/anathema/blob/dev/assets/markdown.gif?raw=true)

#### Screenshot by Jyn
![Bunny rpg by Jyn](https://github.com/togglebyte/anathema/blob/dev/assets/anathema.png?raw=true)

#### Twitch UI by Twitch user s9tpepper_
![Twitch ui by Twitch user s9tpepper_](https://github.com/togglebyte/anathema/blob/dev/assets/twitch-ui.webp?raw=true)


