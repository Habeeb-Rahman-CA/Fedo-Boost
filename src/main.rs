mod app;
mod system;
mod diagnosis;
mod actions;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run_app().await
}
