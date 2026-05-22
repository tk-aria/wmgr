mod presentation;

use presentation::cli::CliApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = CliApp::new();
    app.run().await
}
