mod domain;
mod application;
mod infrastructure;
mod presentation;
mod common;

use presentation::cli::CliApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Run the CLI application
    let app = CliApp::new();
    app.run().await
}