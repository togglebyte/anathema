use std::sync::atomic::{AtomicBool, Ordering};

use anathema_backend::Backend;
use anathema_templates::Document;
use anathema_templates::error::Error as TemplateError;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, recommended_watcher};

use super::Runtime;
use crate::error::{Error, Result};

static STOP_ERROR_RT: AtomicBool = AtomicBool::new(false);

// TODO:
// this should be cleaned up.
// The file watcher is almost identitcal to the one in the runtime.
pub(crate) fn show_error<B: Backend>(error: TemplateError, backend: &mut B, document: &mut Document) -> Result<()> {
    if STOP_ERROR_RT.load(Ordering::Relaxed) {
        panic!("recursive error displays.");
    }

    let tpl = format!(
        "
align [alignment: 'centre']
    border [background: 'red', foreground: 'black']
        vstack
            text [bold: true] 'Template error:'
            text '{error}'
    "
    );

    let doc = Document::new(tpl);

    // File watchers here
    let _watcher = set_watcher(document)?;

    let mut builder = Runtime::builder(doc, backend);
    builder.disable_hot_reload();
    builder.finish(backend, |runtime, backend| {
        runtime.with_frame(backend, |backend, mut frame| {
            loop {
                if STOP_ERROR_RT.load(Ordering::Relaxed) {
                    break;
                }

                frame.tick(backend)?;
                frame.present(backend);
                frame.cleanup();
            }
            backend.clear();
            Err(Error::Stop)
        })
    })?;
    document.reload_templates()?;

    // Reset
    STOP_ERROR_RT.store(false, Ordering::Relaxed);

    Ok(())
}

fn set_watcher(document: &Document) -> Result<RecommendedWatcher> {
    let paths = document
        .template_paths()
        .filter_map(|p| p.canonicalize().ok())
        .collect::<Vec<_>>();

    let mut watcher = recommended_watcher(move |event: std::result::Result<Event, _>| match event {
        Ok(event) => match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Remove(_) | notify::EventKind::Modify(_) => {
                if paths.iter().any(|p| event.paths.contains(p)) {
                    STOP_ERROR_RT.store(true, Ordering::Relaxed);
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => (),
        },
        Err(_err) => (),
    })?;

    for path in document.template_paths() {
        let path = path.canonicalize().unwrap();

        if let Some(parent) = path.parent() {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
        }
    }

    Ok(watcher)
}
