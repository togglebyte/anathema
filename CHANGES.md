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
