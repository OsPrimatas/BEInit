use crate::project_manager::read_cfg::find_project_root;
use crate::utils::bei_paths::BeiPaths;
use crate::utils::bei_props::{BeiProps, DbProps, MariaDbProps, PhpProps};
use std::process::{Child, Command};

#[allow(dead_code)]
pub struct RunningServices {
    pub mariadb: Option<Child>,
    pub php: Option<Child>,
    pub bun: Option<Child>,
}

impl Drop for RunningServices {
    fn drop(&mut self) {
        println!("Parando serviços...");
        if let Some(mut child) = self.mariadb.take() {
            let _ = child.kill();
        }
        if let Some(mut child) = self.php.take() {
            let _ = child.kill();
        }
        if let Some(mut child) = self.bun.take() {
            let _ = child.kill();
        }
        clear_pids_file();
    }
}

fn save_pids(mariadb_pid: u32, php_pid: u32, bun_pid: u32) {
    if let Some(root) = find_project_root() {
        let bei_dir = root.join(".bei");
        if !bei_dir.exists() {
            let _ = std::fs::create_dir_all(&bei_dir);
        }
        let pids_file = bei_dir.join("pids.json");
        let content = format!(
            "{{\n  \"mariadb\": {},\n  \"php\": {},\n  \"bun\": {}\n}}",
            mariadb_pid, php_pid, bun_pid
        );
        let _ = std::fs::write(pids_file, content);
    }
}

fn clear_pids_file() {
    if let Some(root) = find_project_root() {
        let pids_file = root.join(".bei").join("pids.json");
        if pids_file.exists() {
            let _ = std::fs::remove_file(pids_file);
        }
    }
}

pub fn stop_services() -> Result<(), Box<dyn std::error::Error>> {
    let root = match find_project_root() {
        Some(r) => r,
        None => {
            return Err("Nenhum projeto bei detectado nesta pasta.".into());
        }
    };

    let pids_file = root.join(".bei").join("pids.json");
    if !pids_file.exists() {
        println!("Nenhum serviço bei ativo registrado.");
        return Ok(());
    }

    let content = std::fs::read_to_string(&pids_file)?;
    let pids: serde_json::Value = serde_json::from_str(&content)?;

    let tools = ["mariadb", "php", "bun"];
    for tool in &tools {
        if let Some(pid) = pids[tool].as_u64() {
            println!("Finalizando processo {} (PID: {})...", tool, pid);
            kill_process(pid as u32);
        }
    }

    let _ = std::fs::remove_file(pids_file);
    println!("Serviços parados com sucesso.");
    Ok(())
}

fn kill_process(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output();
    }
}

pub async fn start_all(
    config: &BeiProps,
    db: &DbProps,
    paths: &BeiPaths,
) -> Result<RunningServices, Box<dyn std::error::Error>> {
    println!("Iniciando serviços bei...");

    // 1. MariaDB
    let mariadb_child = start_mariadb(&config.mariadb, db, paths).await?;

    // 2. PHP Built-in Server
    let php_child = start_php(&config.php, &config.project_config.backend_path, paths)?;

    // 3. Bun (frontend)
    let bun_child = start_bun(&config.project_config.frontend_path, paths)?;

    let mariadb_pid = mariadb_child.id();
    let php_pid = php_child.id();
    let bun_pid = bun_child.id();
    save_pids(mariadb_pid, php_pid, bun_pid);

    Ok(RunningServices {
        mariadb: Some(mariadb_child),
        php: Some(php_child),
        bun: Some(bun_child),
    })
}

fn start_php(
    php_config: &PhpProps,
    backend_path: &str,
    paths: &BeiPaths,
) -> Result<Child, Box<dyn std::error::Error>> {
    let php_dir = paths.get_php_dir(&php_config.version);
    let php_bin = paths
        .find_executable(&php_dir, "php")
        .unwrap_or_else(|| std::path::PathBuf::from("php"));

    let backend_path_buf = std::path::PathBuf::from(backend_path);
    if !backend_path_buf.exists() {
        std::fs::create_dir_all(&backend_path_buf)?;
    }
    let backend_abs = std::fs::canonicalize(backend_path_buf)?;
    let backend_clean = std::path::PathBuf::from(backend_abs.to_string_lossy().replace("\\\\?\\", ""));

    println!("Iniciando PHP em http://localhost:{}", php_config.port);

    let child = Command::new(php_bin)
        .args([
            "-S",
            &format!("0.0.0.0:{}", php_config.port),
            "-t",
            backend_clean.to_str().unwrap(),
        ])
        .current_dir(backend_clean)
        .spawn()?;

    Ok(child)
}

fn start_bun(frontend_path: &str, paths: &BeiPaths) -> Result<Child, Box<dyn std::error::Error>> {
    let frontend_path_buf = std::path::PathBuf::from(frontend_path);
    if !frontend_path_buf.exists() {
        std::fs::create_dir_all(&frontend_path_buf)?;
    }
    let frontend_abs = std::fs::canonicalize(frontend_path_buf)?;
    let frontend_clean = std::path::PathBuf::from(frontend_abs.to_string_lossy().replace("\\\\?\\", ""));

    let bun_dir = paths.get_bun_dir("latest");
    let bun_bin = paths
        .find_executable(&bun_dir, "bun")
        .unwrap_or_else(|| std::path::PathBuf::from("bun"));

    println!("Iniciando Bun no frontend...");

    let child = Command::new(bun_bin)
        .args(["run", "dev"])
        .current_dir(frontend_clean)
        .spawn()?;

    Ok(child)
}

async fn start_mariadb(
    mariadb_config: &MariaDbProps,
    db: &DbProps,
    paths: &BeiPaths,
) -> Result<Child, Box<dyn std::error::Error>> {
    let mariadb_dir = paths.get_mariadb_dir(&mariadb_config.version);
    let mariadb_bin = paths
        .find_executable(&mariadb_dir, "mariadbd")
        .or_else(|| paths.find_executable(&mariadb_dir, "mysqld"))
        .ok_or_else(|| {
            format!(
                "Não foi possível encontrar o executável do mariadbd para a versão {}",
                mariadb_config.version
            )
        })?;

    println!(
        "Iniciando MariaDB v{} na porta {}",
        mariadb_config.version, mariadb_config.port
    );

    let current_dir = std::env::current_dir()?;
    let data_dir_path = current_dir.join(&mariadb_config.data_dir);
    if !data_dir_path.exists() {
        std::fs::create_dir_all(&data_dir_path)?;
    }

    let mysql_system_db = data_dir_path.join("mysql");
    if !mysql_system_db.exists() {
        println!("Inicializando diretório de dados do MariaDB...");
        let install_bin = paths
            .find_executable(&mariadb_dir, "mariadb-install-db")
            .or_else(|| paths.find_executable(&mariadb_dir, "mysql_install_db"))
            .ok_or_else(|| {
                format!(
                    "Não foi possível encontrar o executável de instalação do MariaDB na versão {}",
                    mariadb_config.version
                )
            })?;

        let mut install_cmd = Command::new(install_bin);
        // Evitar caminhos UNC (\\?\) que o canonicalize gera no Windows
        let datadir_str = data_dir_path.to_string_lossy().replace("\\\\?\\", "");
        install_cmd.args([
            &format!("--datadir={}", datadir_str),
        ]);

        let status = install_cmd.status()?;
        if !status.success() {
            return Err("Falha ao inicializar o banco de dados do MariaDB".into());
        }
    }

    // Criar o init.sql para garantir usuário, senha e banco de dados
    let init_sql_path = data_dir_path.join("init.sql");
    let init_sql_content = format!(
        "CREATE DATABASE IF NOT EXISTS `{}`;\n\
         CREATE USER IF NOT EXISTS '{}'@'localhost' IDENTIFIED BY '{}';\n\
         ALTER USER '{}'@'localhost' IDENTIFIED BY '{}';\n\
         GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'localhost';\n\
         GRANT ALL PRIVILEGES ON *.* TO '{}'@'localhost' WITH GRANT OPTION;\n\
         FLUSH PRIVILEGES;\n",
        db.database, db.user, db.password, db.user, db.password, db.database, db.user, db.user
    );
    std::fs::write(&init_sql_path, init_sql_content)?;

    let mut cmd = Command::new(mariadb_bin);
    let datadir_str = data_dir_path.to_string_lossy().replace("\\\\?\\", "");
    let init_sql_str = init_sql_path.to_string_lossy().replace("\\\\?\\", "");
    
    cmd.args([
        "--port",
        &mariadb_config.port.to_string(),
        &format!("--datadir={}", datadir_str),
        &format!("--init-file={}", init_sql_str),
        "--console",
    ]);

    let child = cmd.spawn()?;
    Ok(child)
}
