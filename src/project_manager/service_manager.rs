use crate::utils::beinit_paths::BEInitPaths;
use crate::utils::beinit_props::{BEInitDbProps, BEInitProps, MariaDbProps, PhpProps};
use std::process::{Child, Command};

#[allow(dead_code)]
pub struct RunningServices {
    pub mariadb: Option<Child>,
    pub php: Option<Child>,
    pub bun: Option<Child>,
}

pub async fn start_all(
    config: &BEInitProps,
    db_config: &BEInitDbProps,
    paths: &BEInitPaths,
) -> Result<RunningServices, Box<dyn std::error::Error>> {
    println!("🚀 Iniciando serviços Beinit...");

    // 1. MariaDB
    let mariadb_child = start_mariadb(&config.mariadb, db_config, paths).await?;

    // 2. PHP Built-in Server
    let php_child = start_php(&config.php, &config.project_config.backend_path, paths)?;

    // 3. Bun (frontend)
    let bun_child = start_bun(&config.project_config.frontend_path, paths)?;

    Ok(RunningServices {
        mariadb: Some(mariadb_child),
        php: Some(php_child),
        bun: Some(bun_child),
    })
}

fn start_php(
    php_config: &PhpProps,
    backend_path: &str,
    paths: &BEInitPaths,
) -> Result<Child, Box<dyn std::error::Error>> {
    let php_dir = paths.get_php_dir(&php_config.version);
    let php_bin = paths
        .find_executable(&php_dir, "php")
        .unwrap_or_else(|| std::path::PathBuf::from("php"));

    let backend_abs = std::fs::canonicalize(backend_path)?;

    println!("🌐 Iniciando PHP em http://localhost:{}", php_config.port);

    let child = Command::new(php_bin)
        .args([
            "-S",
            &format!("0.0.0.0:{}", php_config.port),
            "-t",
            backend_abs.to_str().unwrap(),
        ])
        .current_dir(backend_abs)
        .spawn()?;

    Ok(child)
}

fn start_bun(
    frontend_path: &str,
    paths: &BEInitPaths,
) -> Result<Child, Box<dyn std::error::Error>> {
    let frontend_abs = std::fs::canonicalize(frontend_path)?;

    let bun_dir = paths.get_bun_dir("latest");
    let bun_bin = paths
        .find_executable(&bun_dir, "bun")
        .unwrap_or_else(|| std::path::PathBuf::from("bun"));

    println!("⚡ Iniciando Bun no frontend...");

    let child = Command::new(bun_bin)
        .args(["run", "dev"])
        .current_dir(frontend_abs)
        .spawn()?;

    Ok(child)
}

async fn start_mariadb(
    mariadb_config: &MariaDbProps,
    _db_config: &BEInitDbProps,
    paths: &BEInitPaths,
) -> Result<Child, Box<dyn std::error::Error>> {
    let mariadb_dir = paths.get_mariadb_dir(&mariadb_config.version);
    let mariadb_bin = paths
        .find_executable(&mariadb_dir, "mariadbd")
        .ok_or_else(|| {
            format!(
                "Não foi possível encontrar o executável do mariadbd para a versão {}",
                mariadb_config.version
            )
        })?;

    println!(
        "🗄️  Iniciando MariaDB v{} na porta {}",
        mariadb_config.version, mariadb_config.port
    );

    let mut cmd = Command::new(mariadb_bin);
    cmd.args([
        "--port",
        &mariadb_config.port.to_string(),
        "--datadir",
        &mariadb_config.data_dir,
    ]);

    let child = cmd.spawn()?;
    Ok(child)
}
