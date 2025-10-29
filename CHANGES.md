* 0.2.12
    * Clamping is now true by default on overflow
    * Globals can now be registered with the runtime
    * `attributes.value_as::<T>` is now available on `Attributes`
    * Performance improvements
    * New template function: `len`
    * `List<T>` now has `clear` and `retain`
* 0.2.11
    * FEATURE: ranges 
    * `padding` function
    * `width` function
* 0.2.10
    * BUGFIX: `on_mount` is now called after the children are generated
    * BUGFIX: `on_tick` is now run before the cycle call 
    * BUGFIX: `expand` would make the constraints tight, this is no longer the
      case
* 0.2.9
    * New function: truncate
    * New border style: "rounded"
    * Hex and Color can now be serialized / deserialized if the `serde` feature
      is enabled.
    * `MessageReceiver` is now part of prelude
* 0.2.8
    * An emitter can be created before the runtime
    * Messages can be emitted to both widget ids and component ids
    * Feature flag: `serde` is now a feature flag that adds `Serialize` / `Deserialize` to `WidgetId`
    * There is now a distinction between `global`s and `local`s
    * BUGFIX: ctrl+c works with the error display
    * Trying to use a component twice will now include the component name in the
      error
    * Global definitions will raise an error if it's already assigned
* 0.2.7
    * BUGFIX: use correct truthiness check in control flow update
* 0.2.6
    * `Either` now works by doing truthiness checks on state
    * `Backend::full_screen()` convenience function
    * `to_float` template function
    * `PathBuf`s can now be used as templates
    * BUGFIX: tick events no longer tries to use removed components
    * BUGFIX: erasing characters correctly between frames
* 0.2.5
    * BUGFIX: component reuse in if / else
    * `with` statement
* 0.2.4
    * Switch / case / default
    * BUGFIX: Tuple structs now works as state
    * BUGFIX: Resolving an index in the template no longer panics if it's
      outside of the range
    * `add_function` renamed to `register_function`
    * `register_function` now returns an error if the function exists
* 0.2.3
    * Templates now has functions
        * to_upper
        * to_lower
        * to_str
        * to_int
        * round
        * contains
    * BUGFIX: component tick event happened after the widget cycle causing a
      panic
* 0.2.2
    * Everything: this is a complete rewrite
