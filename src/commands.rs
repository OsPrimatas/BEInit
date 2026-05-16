use clap::{Parser, Subcommand};
use std::error::Error;

use crate::download_manager::bun_download;
use crate::download_manager::mariadb_download;
use crate::download_manager::php_download;
use crate::project_manager::read_cfg::load_configs;
use crate::project_manager::service_manager;
use crate::utils::beinit_paths::BEInitPaths;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Beinit - Gerenciador local de ambiente PHP + MariaDB"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Install,
    Run,
    Stop,
    Status,
    Php {
        #[command(subcommand)]
        action: PhpCommands,
    },
}

#[derive(Subcommand)]
pub enum PhpCommands {
    Use { version: String },
    List,
}

// Função principal dos comandos
pub async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("🚀 Inicializando projeto...");
            init_project()?;
        }
        Commands::Install => {
            println!("🛠️  Instalando dependências do projeto...");
            let (config, _db_config) = load_configs()?;
            let paths = BEInitPaths::new();
            paths.ensure_dirs_exist()?;

            // Download PHP
            php_download::download_php_if_needed(&config.php.version, &paths).await?;

            // Download MariaDB
            mariadb_download::download_mariadb_if_needed(&config.mariadb.version, &paths).await?;

            // Download Bun
            bun_download::download_bun_if_needed(&paths).await?;

            println!("✨ Instalação concluída com sucesso!");
        }
        Commands::Run => {
            println!("▶️  Iniciando serviços...");
            let (config, db_config) = load_configs()?;
            let paths = BEInitPaths::new();

            let _services = service_manager::start_all(&config, &db_config, &paths).await?;

            println!("✅ Todos os serviços iniciados!");
            println!("🌐 PHP       -> http://localhost:{}", config.php.port);
            println!("🗄️  MariaDB   -> porta {}", config.mariadb.port);
            println!("⚡ Bun       -> frontend dev server");

            tokio::signal::ctrl_c().await?;
            println!("\n⏹️  Recebido Ctrl+C, parando serviços...");
        }
        Commands::Stop => println!("⏹️  Parando serviços..."),
        Commands::Status => println!("📊 Status atual..."),
        Commands::Php { action } => match action {
            PhpCommands::Use { version } => println!("🔄 Mudando para PHP {}", version),
            PhpCommands::List => println!("📜 Listando versões do PHP..."),
        },
    }
    Ok(())
}

fn init_project() -> Result<(), Box<dyn Error>> {
    let cfg_path = "beinit.cfg.json";
    let db_path = "beinit.db.json";

    if !std::path::Path::new(cfg_path).exists() {
        let default_cfg = r#"{
    "frontend_path": "./frontend",
    "backend_path": "./backend",
    "php": {
        "version": "8.3.6",
        "port": 8000,
        "display_errors": true
    },
    "mariadb": {
        "version": "11.4.2",
        "port": 3306,
        "data_dir": "./data/mysql"
    }
}"#;
        std::fs::write(cfg_path, default_cfg)?;
        println!("✅ Arquivo beinit.cfg.json criado!");
    } else {
        println!("ℹ️  Arquivo beinit.cfg.json já existe.");
    }

    if !std::path::Path::new(db_path).exists() {
        let default_db = r#"{
    "mariadb_user": "root",
    "mariadb_password": "",
    "mariadb_database": "beinit_db"
}"#;
        std::fs::write(db_path, default_db)?;
        println!("✅ Arquivo beinit.db.json criado!");
    } else {
        println!("ℹ️  Arquivo beinit.db.json já existe.");
    }

    Ok(())
}
