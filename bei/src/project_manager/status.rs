use crate::project_manager::read_cfg::{load_configs, load_db_config, find_project_root};
use crate::utils::bei_paths::BeiPaths;
use colored::Colorize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

async fn is_port_open(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    if let Ok(socket_addr) = addr.parse::<std::net::SocketAddr>() {
        match timeout(
            Duration::from_millis(300),
            TcpStream::connect(&socket_addr),
        )
        .await
        {
            Ok(Ok(_)) => true,
            _ => false,
        }
    } else {
        false
    }
}

pub async fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "======================================================".cyan().bold());
    println!("{}", "               STATUS DO AMBIENTE BEI                 ".cyan().bold());
    println!("{}", "======================================================".cyan().bold());
    println!();

    // 1. Verificar se o projeto está inicializado
    let root = match find_project_root() {
        Some(r) => r,
        None => {
            println!("{}", "Nenhum projeto bei detectado nesta pasta.".red().bold());
            println!("Para inicializar um novo projeto, execute: {}", "bei init".yellow());
            return Ok(());
        }
    };

    let config = match load_configs() {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erro ao carregar as configurações do projeto:".red().bold(), e);
            return Ok(());
        }
    };

    let db_config = load_db_config().ok();

    // --- CONFIGURAÇÃO E ESTRUTURA DO PROJETO ---
    println!("{}", "--- CONFIGURAÇÃO E ESTRUTURA ---".blue().bold());
    
    let cfg_json_path = root.join("bei.cfg.json");
    print_file_status("Configuração Principal (bei.cfg.json)", cfg_json_path.exists());

    let frontend_path = root.join(&config.project_config.frontend_path);
    print_file_status(&format!("Pasta Frontend ({})", config.project_config.frontend_path), frontend_path.exists());

    let backend_path = root.join(&config.project_config.backend_path);
    print_file_status(&format!("Pasta Backend ({})", config.project_config.backend_path), backend_path.exists());

    let gitignore_path = root.join(".gitignore");
    print_file_status("Arquivo .gitignore", gitignore_path.exists());

    let env_path = backend_path.join(".env");
    print_file_status("Arquivo de Ambiente (backend/.env)", env_path.exists());

    let db_json_path = backend_path.join("bei.db.json");
    print_file_status("Configuração do Banco (backend/bei.db.json)", db_json_path.exists());
    
    println!();

    // --- STATUS DAS FERRAMENTAS INSTALADAS ---
    println!("{}", "--- FERRAMENTAS INSTALADAS ---".blue().bold());
    let paths = BeiPaths::new();

    // PHP
    let php_dir = paths.get_php_dir(&config.php.version);
    let php_exe = paths.find_executable(&php_dir, "php");
    print_tool_status("PHP", &config.php.version, php_exe.is_some(), php_exe);

    // MariaDB
    let mariadb_dir = paths.get_mariadb_dir(&config.mariadb.version);
    let mariadb_exe = paths.find_executable(&mariadb_dir, "mariadbd")
        .or_else(|| paths.find_executable(&mariadb_dir, "mysqld"));
    print_tool_status("MariaDB", &config.mariadb.version, mariadb_exe.is_some(), mariadb_exe);

    // Bun
    let bun_dir = paths.get_bun_dir("latest");
    let bun_exe = paths.find_executable(&bun_dir, "bun");
    print_tool_status("Bun", &config.bun.version, bun_exe.is_some(), bun_exe);

    // Composer
    let composer_dir = paths.bin_dir.join("composer").join(&config.composer.version);
    let composer_exe = paths.find_executable(&composer_dir, "composer");
    print_tool_status("Composer", &config.composer.version, composer_exe.is_some(), composer_exe);

    println!();

    // --- STATUS DE EXECUÇÃO DOS SERVIÇOS ---
    println!("{}", "--- SERVIÇOS EM EXECUÇÃO ---".blue().bold());

    // PHP Dev Server
    let php_active = is_port_open(config.php.port).await;
    print_service_status("PHP Dev Server", config.php.port, php_active, &format!("http://localhost:{}", config.php.port));

    // MariaDB
    let mariadb_active = is_port_open(config.mariadb.port).await;
    let db_name = db_config.as_ref().map(|d| d.database.as_str()).unwrap_or("bei_db");
    print_service_status("MariaDB Server", config.mariadb.port, mariadb_active, &format!("localhost:{} (Banco: {})", config.mariadb.port, db_name));

    // Frontend (Vite/Bun) - Porta padrão 5173
    let frontend_port = 5173;
    let frontend_active = is_port_open(frontend_port).await;
    print_service_status("Frontend (Vite/Bun)", frontend_port, frontend_active, &format!("http://localhost:{}", frontend_port));

    println!();
    println!("{}", "======================================================".cyan().bold());
    Ok(())
}

fn print_file_status(label: &str, exists: bool) {
    if exists {
        println!("{:<45} [{}]", label, "OK".green().bold());
    } else {
        println!("{:<45} [{}]", label, "AUSENTE".red().bold());
    }
}

fn print_tool_status(name: &str, version: &str, installed: bool, path: Option<std::path::PathBuf>) {
    let status_str = if installed {
        "Instalado".green().bold()
    } else {
        "Não Instalado".red().bold()
    };

    println!("{:<12} (v{:<8}) -> {}", name, version, status_str);
    if let Some(p) = path {
        println!("   Caminho: {}", p.display().to_string().dimmed());
    }
}

fn print_service_status(name: &str, port: u16, active: bool, info: &str) {
    let status_str = if active {
        "ATIVO".green().bold()
    } else {
        "INATIVO".red().bold()
    };

    println!("{:<22} (Porta {:<5}) -> {} ({})", name, port, status_str, info.dimmed());
}
