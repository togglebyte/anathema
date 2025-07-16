use std::fs::read_to_string;

use anathema::runtime::Runtime;
use anathema::templates::{Document, ToSourceKind};
use anathema_ssh::sshserver::AnathemaSSHServer;
use anyhow;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut server = AnathemaSSHServer::new(move |mut backend| {
        let template = read_to_string("examples/templates/basic/basic.aml").unwrap();
        let doc = Document::new("@index");
        let mut builder = Runtime::builder(doc, &backend);
        builder.template("index", template.to_template()).unwrap();
        builder
            .finish(&mut backend, |runtime, backend| {
                println!("RUNTIME RUN...");
                runtime.run(backend)
            })
            .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))
    });

    println!("Starting SSH server...");
    server.run().await?;
    println!("SSH server stopped.");
    Ok(())
}
