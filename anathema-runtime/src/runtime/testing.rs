use std::time::{Duration, Instant};

use anathema_backend::Backend;
use anathema_state::{drain_watchers, Watched, Watcher};
use anathema_store::stack::Stack;
use anathema_widgets::query::Children;

use crate::Frame;
use crate::error::Result;

// -----------------------------------------------------------------------------
//   - Used with test driver -
//   These functions should not be used outside of testing
// -----------------------------------------------------------------------------
impl<'bp> Frame<'_, 'bp> {
    pub fn components(&mut self) -> anathema_widgets::query::Components<'_, '_, 'bp> {
        panic!()
        // anathema_widgets::query::Components::new(
        //     self.tree.view_mut(),
        //     self.layout_ctx.attribute_storage,
        //     self.layout_ctx.dirty_widgets,
        // )
    }

    pub fn elements(&mut self) -> Children<'_, 'bp> {
        Children::new(
            self.tree.view_mut(),
            self.layout_ctx.attribute_storage,
            self.layout_ctx.dirty_widgets,
        )
    }

    // TODO: this can't really be called a frame if we can tick it multiple
    // times. Maybe RuntimeMut or something less mental
    pub fn wait_for_monitor<B: Backend>(
        &mut self,
        backend: &mut B,
        watcher: Watcher,
        mut timeout: Duration,
    ) -> Result<Watched> {
        let now = Instant::now();

        let mut watchers = Stack::empty();
        drain_watchers(&mut watchers);

        if watchers.contains(&watcher) {
            return Ok(Watched::Triggered);
        }

        loop {
            let dur = self.tick(backend, false);
            self.present(backend);
            self.cleanup();

            drain_watchers(&mut watchers);

            if watchers.contains(&watcher) {
                return Ok(Watched::Triggered);
            }

            if timeout.saturating_sub(now.elapsed()).is_zero() {
                break Ok(Watched::Timeout);
            }

            let sleep = self.sleep_micros - dur.as_micros();
            std::thread::sleep(Duration::from_micros(sleep as u64));
        }
    }
}
