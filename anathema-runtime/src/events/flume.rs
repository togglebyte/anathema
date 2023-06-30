use super::{Event, EventProvider};

/// An event provider based on a flume channel.
pub struct FlumeEventProvider {
    rx: flume::Receiver<Event>,
}

impl FlumeEventProvider {
    pub fn with_capacity(cap: usize) -> (flume::Sender<Event>, Self) {
        let (tx, rx) = flume::bounded(cap);

        let inst = Self { rx };

        (tx, inst)
    }
}

impl EventProvider for FlumeEventProvider {
    fn next(&mut self) -> Option<Event> {
        self.rx.recv().ok()
    }
}
