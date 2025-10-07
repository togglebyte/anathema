use std::sync::atomic::{AtomicBool, Ordering};

use anathema_backend::Backend;
use anathema_templates::Document;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, recommended_watcher};

use super::Runtime;
use crate::error::{Error, Result};

static STOP_ERROR_RT: AtomicBool = AtomicBool::new(false);

// TODO:
// this should be cleaned up.
// The file watcher is almost identitcal to the one in the runtime.
pub(crate) fn show_error<B: Backend>(error: Error, backend: &mut B, document: &Document) -> Result<()> {
    if STOP_ERROR_RT.load(Ordering::Relaxed) {
        panic!("recursive error displays.");
    }

    let template_path = match &error {
        Error::Template(error) => format!(": {}", error.path()),
        Error::Widget(error) => match error.path.as_ref() {
            Some(path) => format!(": {}", path.display()),
            None => String::new(),
        },
        _ => String::new(),
    };

    let tpl = format!(
        "
align [alignment: 'centre']
    border [background: 'red', foreground: 'black']
        vstack
            text [bold: true] 'Template error{template_path}'
            text '{error}'
    "
    );

    let doc = Document::new(tpl);

    // File watchers here
    let _watcher = set_watcher(document);

    let mut builder = Runtime::builder(doc, backend);
    builder.hot_reload(false);
    let res = builder.finish(backend, |runtime, backend| {
        runtime.with_frame(backend, |backend, mut frame| {
            loop {
                if STOP_ERROR_RT.load(Ordering::Relaxed) {
                    break Err(Error::Reload);
                }

                frame.tick(backend)?;
                frame.present(backend);
                frame.cleanup();
            }?;
            backend.clear();
            Err(Error::Stop)
        })
    });

    // Reset
    STOP_ERROR_RT.store(false, Ordering::Relaxed);

    res
}

fn set_watcher(document: &Document) -> Result<RecommendedWatcher> {
    let _paths = document
        .template_paths()
        .filter_map(|p| p.canonicalize().ok())
        .collect::<Vec<_>>();

    let mut watcher = recommended_watcher(move |event: std::result::Result<Event, _>| match event {
        Ok(event) => match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Remove(_) | notify::EventKind::Modify(_) => {
                // NOTE: we'll let any changes to any files stop the error runtime,
                // and let the main runtime attempt to restart again.
                //
                // This might change in the future but it works for now

                // if paths.iter().any(|p| event.paths.contains(p)) {
                STOP_ERROR_RT.store(true, Ordering::Relaxed);
                // }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => (),
        },
        Err(_err) => (),
    })?;

    for path in document.template_paths() {
        let Some(path) = path.parent() else { continue };
        let Ok(path) = path.canonicalize() else { continue };

        watcher.watch(&path, RecursiveMode::NonRecursive)?;
    }

    Ok(watcher)
}
