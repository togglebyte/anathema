* 0.1.2
    * `WidgetContainer::by_id` is made more ergonomic and can now called with a
      string slice now.
    * BUGFIX: Disabling mouse capture by default under `Windows` caused a panic,
      this is no longer done.
    * `metrics` is now a feature (turned on by default), that always updates the
      context with the last frame numbers
