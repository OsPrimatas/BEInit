mod commands;
mod download_manager;
mod project_manager;
mod utils;

use commands::run;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("❌ Erro: {}", e);
        std::process::exit(1);
    }
}
