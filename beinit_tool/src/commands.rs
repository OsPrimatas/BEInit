use clap::{Parser, Subcommand};
use std::error::Error;

use crate::download_manager::bun_download;
use crate::download_manager::composer_download;
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
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Bun {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Composer {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
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
            let config = load_configs()?;
            let paths = BEInitPaths::new();
            paths.ensure_dirs_exist()?;

            // Download PHP
            php_download::download_php_if_needed(&config.php.version, &paths).await?;

            // Download MariaDB
            mariadb_download::download_mariadb_if_needed(&config.mariadb.version, &paths).await?;

            // Download Bun
            bun_download::download_bun_if_needed(&paths).await?;

            // Download Composer
            composer_download::download_composer_if_needed(&config.composer.version, &paths)
                .await?;

            println!("✨ Instalação concluída com sucesso!");
        }
        Commands::Run => {
            println!("▶️  Iniciando serviços...");
            let config = load_configs()?;
            let paths = BEInitPaths::new();

            let _services = service_manager::start_all(&config, &paths).await?;

            println!("✅ Todos os serviços iniciados!");
            println!("🌐 PHP       -> http://localhost:{}", config.php.port);
            println!("🗄️  MariaDB   -> porta {}", config.mariadb.port);
            println!("⚡ Bun       -> frontend dev server");

            tokio::signal::ctrl_c().await?;
            println!("\n⏹️  Recebido Ctrl+C, parando serviços...");
        }
        Commands::Stop => println!("⏹️  Parando serviços..."),
        Commands::Status => println!("📊 Status atual..."),
        Commands::Php { args } => {
            let config = load_configs()?;
            let paths = BEInitPaths::new();
            let php_dir = paths.ensure_version_dir("php", &config.php.version)?;
            let php_exe = paths
                .find_executable(&php_dir, "php")
                .expect("PHP não encontrado. Execute 'beinit install' primeiro.");

            std::process::Command::new(php_exe).args(args).status()?;
        }
        Commands::Bun { args } => {
            let config = load_configs()?;
            let paths = BEInitPaths::new();
            let bun_dir = paths.ensure_version_dir("bun", &config.bun.version)?;
            let bun_exe = paths
                .find_executable(&bun_dir, "bun")
                .expect("Bun não encontrado. Execute 'beinit install' primeiro.");

            std::process::Command::new(bun_exe).args(args).status()?;
        }
        Commands::Composer { args } => {
            let config = load_configs()?;
            let paths = BEInitPaths::new();
            let composer_dir = paths.ensure_version_dir("composer", &config.composer.version)?;
            let composer_exe = paths
                .find_executable(&composer_dir, "composer")
                .expect("Composer não encontrado. Execute 'beinit install' primeiro.");

            // We need PHP in the PATH or explicitly use it, but our composer.bat already calls php
            // However, php might not be in PATH. Let's make sure it is in PATH.
            let php_dir = paths.ensure_version_dir("php", &config.php.version)?;

            let mut cmd = std::process::Command::new(composer_exe);
            let current_path = std::env::var("PATH").unwrap_or_default();
            cmd.env("PATH", format!("{};{}", php_dir.display(), current_path));

            cmd.args(args).status()?;
        }
    }
    Ok(())
}

fn init_project() -> Result<(), Box<dyn Error>> {
    let cfg_path = "beinit.cfg.json";

    // Ler os templates embutidos
    let default_cfg = include_str!("../assets/beinit.cfg.json");
    let default_env = include_str!("../assets/beinit.db.env");

    // Se o arquivo cfg_path já existe, lê o que está no arquivo. Caso contrário, cria o default e usa.
    let cfg_content = if std::path::Path::new(cfg_path).exists() {
        println!("ℹ️  Arquivo beinit.cfg.json já existe. Usando configurações existentes.");
        std::fs::read_to_string(cfg_path)?
    } else {
        std::fs::write(cfg_path, default_cfg)?;
        println!("✅ Arquivo beinit.cfg.json criado!");
        default_cfg.to_string()
    };

    let parsed_cfg: serde_json::Value = serde_json::from_str(&cfg_content)?;
    let frontend_path = parsed_cfg["project_config"]["frontend_path"]
        .as_str()
        .unwrap_or("frontend");
    let backend_path = parsed_cfg["project_config"]["backend_path"]
        .as_str()
        .unwrap_or("backend");

    let add_gitignore = parsed_cfg["project_config"]["add_gitignore"]
        .as_bool()
        .unwrap_or(true);
    let add_env = parsed_cfg["project_config"]["add_env"]
        .as_bool()
        .unwrap_or(true);
    let add_composer = parsed_cfg["project_config"]["add_composer_file"]
        .as_bool()
        .unwrap_or(true);

    // Criar pastas
    if !std::path::Path::new(frontend_path).exists() {
        std::fs::create_dir_all(frontend_path)?;
        println!("✅ Pasta {} criada!", frontend_path);
    }

    if !std::path::Path::new(backend_path).exists() {
        std::fs::create_dir_all(backend_path)?;
        println!("✅ Pasta {} criada!", backend_path);
    }

    // .gitignore
    if add_gitignore {
        let gitignore_path = ".gitignore";
        let env_entry = format!("{}/.env", backend_path);
        let db_env_entry = format!("{}/beinit.db.env", backend_path);

        if !std::path::Path::new(gitignore_path).exists() {
            let gitignore_content = format!(
                r#"# IDEs e Editores
.idea/
.vscode/
*.swp
*.swo

# Node
{}/node_modules/
{}/dist/
{}/.nx/

# Composer
{}/vendor/

# BEInit e Dados Locais
.beinit/
{}
{}
"#,
                frontend_path, frontend_path, frontend_path, backend_path, env_entry, db_env_entry
            );
            std::fs::write(gitignore_path, gitignore_content)?;
            println!("✅ Arquivo .gitignore criado!");
        } else {
            // Se já existe, garante que .env e beinit.db.env estão lá
            let mut content = std::fs::read_to_string(gitignore_path)?;
            let mut modified = false;

            if !content.contains(&env_entry) {
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!("{}\n", env_entry));
                modified = true;
            }
            if !content.contains(&db_env_entry) {
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!("{}\n", db_env_entry));
                modified = true;
            }

            if modified {
                std::fs::write(gitignore_path, content)?;
                println!("✅ Arquivo .gitignore atualizado com novas exclusões!");
            }
        }
    }

    // .env e beinit.db.env no backend
    if add_env {
        let env_path = format!("{}/.env", backend_path);
        if !std::path::Path::new(&env_path).exists() {
            std::fs::write(&env_path, default_env)?;
            println!("✅ Arquivo .env criado em {}/", backend_path);
        }

        let db_env_path = format!("{}/beinit.db.env", backend_path);
        if !std::path::Path::new(&db_env_path).exists() {
            std::fs::write(&db_env_path, default_env)?;
            println!("✅ Arquivo beinit.db.env criado em {}/", backend_path);
        }
    }

    // composer.json no backend
    if add_composer {
        let composer_path = format!("{}/composer.json", backend_path);
        if !std::path::Path::new(&composer_path).exists() {
            let composer_content = r#"{
    "name": "beinit/backend",
    "description": "Backend project powered by BEInit",
    "type": "project",
    "require": {}
}"#;
            std::fs::write(&composer_path, composer_content)?;
            println!("✅ Arquivo composer.json criado em {}/", backend_path);
        }
    }

    Ok(())
}
