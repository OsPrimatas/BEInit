use clap::{Parser, Subcommand};
use std::error::Error;

use crate::download_manager::bun_download;
use crate::download_manager::composer_download;
use crate::download_manager::mariadb_download;
use crate::download_manager::php_download;
use crate::project_manager::read_cfg::{load_configs, load_db_config};
use crate::project_manager::service_manager;
use crate::utils::bei_paths::BeiPaths;

#[derive(Parser)]
#[command(
    name = "bei",
    author,
    version,
    long_about = None,
    about = "Bei - Gerenciador local de ambiente PHP + MariaDB"
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
            println!("Inicializando projeto...");
            init_project()?;
        }
        Commands::Install => {
            println!("Instalando dependências do projeto...");
            let config = load_configs()?;
            let paths = BeiPaths::new();
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

            println!("Instalação concluída com sucesso!");
        }
        Commands::Run => {
            let config = load_configs()?;
            let db = load_db_config()?;
            let paths = BeiPaths::new();

            let _services = service_manager::start_all(&config, &db, &paths).await?;

            println!("");
            println!("╔══════════════════════════════════════════════════════╗");
            println!("║             SERVIÇOS INICIADOS COM SUCESSO           ║");
            println!("╠══════════════════════════════════════════════════════╣");
            println!("║      PHP Dev Server                                  ║");
            println!(
                "║      URL:   {:<41}║",
                format!("http://localhost:{}", config.php.port)
            );
            println!("╠══════════════════════════════════════════════════════╣");
            println!("║      MariaDB                                         ║");
            println!("║      Host:      {:<37}║", "localhost");
            println!(
                "║      Porta:     {:<37}║",
                config.mariadb.port
            );
            println!(
                "║      Banco:     {:<37}║",
                db.database
            );
            println!(
                "║      Usuário:   {:<37}║",
                db.user
            );
            println!(
                "║      Senha:     {:<37}║",
                db.password
            );
            println!(
                "║      URL JDBC:  {:<37}║",
                format!("jdbc:mariadb://localhost:{}/{}", config.mariadb.port, db.database)
            );
            println!("╠══════════════════════════════════════════════════════╣");
            println!("║      Bun  -  Frontend Dev Server                     ║");
            println!("╚══════════════════════════════════════════════════════╝");
            println!("");
            println!("  Pressione Ctrl+C para parar todos os serviços.");
            println!("");

            tokio::signal::ctrl_c().await?;
            println!("\n⏹️  Recebido Ctrl+C, parando serviços...");
        }
        Commands::Stop => println!("Parando serviços..."),
        Commands::Status => println!("Status atual..."),
        Commands::Php { args } => {
            let config = load_configs()?;
            let paths = BeiPaths::new();
            let php_dir = paths.ensure_version_dir("php", &config.php.version)?;
            let php_exe = match paths.find_executable(&php_dir, "php") {
                Some(exe) => exe,
                None => {
                    eprintln!("❌ PHP não encontrado. Execute 'bei install' primeiro.");
                    std::process::exit(1);
                }
            };

            std::process::Command::new(php_exe).args(args).status()?;
        }
        Commands::Bun { args } => {
            // O download manager instala o Bun sempre na pasta 'latest'
            let paths = BeiPaths::new();
            let bun_dir = paths.ensure_version_dir("bun", "latest")?;
            let bun_exe = match paths.find_executable(&bun_dir, "bun") {
                Some(exe) => exe,
                None => {
                    eprintln!("❌ Bun não encontrado. Execute 'bei install' primeiro.");
                    std::process::exit(1);
                }
            };

            std::process::Command::new(bun_exe).args(args).status()?;
        }
        Commands::Composer { args } => {
            let config = load_configs()?;
            let paths = BeiPaths::new();
            let composer_dir = paths.ensure_version_dir("composer", &config.composer.version)?;
            let composer_exe = match paths.find_executable(&composer_dir, "composer") {
                Some(exe) => exe,
                None => {
                    eprintln!("❌ Composer não encontrado. Execute 'bei install' primeiro.");
                    std::process::exit(1);
                }
            };

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
    let cfg_path = "bei.cfg.json";

    // Ler os templates embutidos
    let default_cfg = include_str!("../assets/bei.cfg.json");
    let default_db_json = include_str!("../assets/bei.db.json");

    // Se o arquivo cfg_path já existe, lê o que está no arquivo. Caso contrário, cria o default e usa.
    let cfg_content = if std::path::Path::new(cfg_path).exists() {
        println!("ℹ️  Arquivo bei.cfg.json já existe. Usando configurações existentes.");
        std::fs::read_to_string(cfg_path)?
    } else {
        std::fs::write(cfg_path, default_cfg)?;
        println!("✅ Arquivo bei.cfg.json criado!");
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
        let db_json_entry = format!("{}/bei.db.json", backend_path);

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

# bei e Dados Locais
.bei/
{}
{}
"#,
                frontend_path, frontend_path, frontend_path, backend_path, env_entry, db_json_entry
            );
            std::fs::write(gitignore_path, gitignore_content)?;
            println!("✅ Arquivo .gitignore criado!");
        } else {
            // Se já existe, garante que .env e bei.db.json estão lá
            let mut content = std::fs::read_to_string(gitignore_path)?;
            let mut modified = false;

            if !content.contains(&env_entry) {
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!("{}\n", env_entry));
                modified = true;
            }
            if !content.contains(&db_json_entry) {
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!("{}\n", db_json_entry));
                modified = true;
            }

            if modified {
                std::fs::write(gitignore_path, content)?;
                println!("✅ Arquivo .gitignore atualizado com novas exclusões!");
            }
        }
    }

    // .env no backend (arquivo genérico de variáveis de ambiente)
    if add_env {
        let env_path = format!("{}/.env", backend_path);
        if !std::path::Path::new(&env_path).exists() {
            let default_env_content = "# Variáveis de ambiente do backend\n# Configure suas chaves de API e outras variáveis aqui\n";
            std::fs::write(&env_path, default_env_content)?;
            println!("✅ Arquivo .env criado em {}/", backend_path);
        }
    }

    // bei.db.json no backend (credenciais do banco de dados) - sempre criado junto do composer.json
    let db_json_path = format!("{}/bei.db.json", backend_path);
    if !std::path::Path::new(&db_json_path).exists() {
        std::fs::write(&db_json_path, default_db_json)?;
        println!("✅ Arquivo bei.db.json criado em {}/", backend_path);
    }

    // composer.json no backend
    if add_composer {
        let composer_path = format!("{}/composer.json", backend_path);
        if !std::path::Path::new(&composer_path).exists() {
            let composer_content = r#"{
    "name": "bei/backend",
    "description": "Backend project powered by bei",
    "type": "project",
    "require": {}
}"#;
            std::fs::write(&composer_path, composer_content)?;
            println!("✅ Arquivo composer.json criado em {}/", backend_path);
        }
    }

    Ok(())
}
