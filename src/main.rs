mod application;
mod common;
mod domain;
mod infrastructure;
mod presentation;

use presentation::cli::CliApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Run the CLI application
    let app = CliApp::new();
    app.run().await
}
