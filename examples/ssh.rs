use anathema::component::{Children, Component, Context};
use anathema::runtime::Runtime;
use anathema::templates::Document;
use anathema_ssh::error::Error;
use anathema_ssh::sshserver::AnathemaSSHServer;
use anathema_state::{State, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut server = AnathemaSSHServer::builder()
        .runtime_factory(|| {
            Box::new(move |backend| {
                let doc = Document::new("@main");
                let mut builder = Runtime::builder(doc, backend);
                builder
                    .component(
                        "main",
                        "examples/templates/ssh/ssh.aml",
                        App,
                        AppState {
                            number: 0.into(),
                            events: 0.into(),
                        },
                    )
                    .unwrap();
                builder.finish(backend, |runtime, backend| {
                    println!("RUNTIME RUN...");
                    runtime.run(backend)
                })
            })
        })
        .enable_mouse()
        .build();

    println!("Starting SSH server...");
    server.run().await?;
    println!("SSH server stopped.");
    Ok(())
}

struct App;

#[derive(State)]
struct AppState {
    number: Value<i32>,
    events: Value<i32>,
}

impl Component for App {
    type Message = ();
    type State = AppState;

    const TICKS: bool = true;

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
        _: std::time::Duration,
    ) {
        *state.number.to_mut() += 1;
    }

    fn on_key(
        &mut self,
        _: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        *state.events.to_mut() += 1;
    }

    fn accept_focus(&self) -> bool {
        true
    }
}
