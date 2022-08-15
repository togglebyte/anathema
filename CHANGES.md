* 0.2.0
    * `ScrollView` widget was removed
    * `DataCtx` now has `get_ref` counter part to `get_mut`.
    * `Border` has an optional `min-width` and `min-height`.
    * `VStack` / `HStack` and `ZStack` now have an optional `min-width` and
      `min-height`.
    * Convenient `String` access on the `Text` widget via
      `get_text_mut(span_index)` and `get_text(span_index)`.
    * BUGFIX: Padding for `ZStack`, `Alignment` and `Expanded` is now working.
    * BUGFIX: padding on `HStack`
    * Rename feature "with-flume" to "flume"
* 0.1.2
    * `WidgetContainer::by_id` is made more ergonomic and can now called with a
      string slice now.
    * BUGFIX: Disabling mouse capture by default under `Windows` caused a panic,
      this was fixed by not disabling mouse capture on Windows by default.
    * `metrics` is now a feature (turned on by default), that always updates the
      context with the last frame numbers.
