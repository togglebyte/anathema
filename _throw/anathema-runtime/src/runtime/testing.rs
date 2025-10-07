use std::time::{Duration, Instant};

use anathema_backend::Backend;
use anathema_state::{Watched, Watcher, drain_watchers};
use anathema_store::stack::Stack;

use crate::Frame;
use crate::error::Result;
use crate::events::GlobalEventHandler;

// -----------------------------------------------------------------------------
//   - Used with test driver -
//   These functions should not be used outside of testing
// -----------------------------------------------------------------------------
impl<'bp, G> Frame<'_, 'bp, G>
where
    G: GlobalEventHandler,
{
    // TODO: Do we need this for anything? - TB 2025-08-27
    // pub fn elements(&mut self) -> Children<'_, 'bp> {
    //     Children::new(
    //         self.tree.view(),
    //         self.layout_ctx.attribute_storage,
    //         &mut self.needs_layout,
    //     )
    // }

    // TODO: this can't really be called a frame if we can tick it multiple
    // times. Maybe RuntimeMut or something less mental
    pub fn wait_for_monitor<B: Backend>(
        &mut self,
        backend: &mut B,
        watcher: Watcher,
        timeout: Duration,
    ) -> Result<Watched> {
        let now = Instant::now();

        let mut watchers = Stack::empty();
        drain_watchers(&mut watchers);

        if watchers.contains(&watcher) {
            return Ok(Watched::Triggered);
        }

        loop {
            let dur = self.tick(backend)?;
            self.present(backend);
            self.cleanup();

            drain_watchers(&mut watchers);

            if watchers.contains(&watcher) {
                return Ok(Watched::Triggered);
            }

            if timeout.saturating_sub(now.elapsed()).is_zero() {
                break Ok(Watched::Timeout);
            }

            let sleep = self.sleep_micros - dur.as_micros() as u64;
            std::thread::sleep(Duration::from_micros(sleep));
        }
    }
}
