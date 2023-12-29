use anathema::values::State;
use anathema::StateValue;

#[derive(Debug, State)]
pub(super) struct Size {
    pub(super) width: StateValue<usize>,
    pub(super) height: StateValue<usize>,
}

#[derive(Debug, State)]
pub(super) struct Meta {
    pub(super) _size: Size,
    pub(super) _timings: Timings,
    pub(super) _focus: StateValue<bool>,
    pub(super) _count: StateValue<usize>,
}

impl Meta {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self {
            _size: Size {
                width: width.into(),
                height: height.into(),
            },
            _timings: Timings::default(),
            _focus: true.into(),
            _count: 0.into(),
        }
    }
}

#[derive(Debug, Default, State)]
pub(super) struct Timings {
    pub(super) layout: StateValue<String>,
    pub(super) position: StateValue<String>,
    pub(super) paint: StateValue<String>,
    pub(super) render: StateValue<String>,
    pub(super) total: StateValue<String>,
}
