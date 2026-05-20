use crate::utils::bei_props::{BeiProps, DbProps};
use std::env;
use std::path::PathBuf;

// Verificar se o arquivo bei.cfg.json existe na raiz do projeto
pub fn find_project_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;

    loop {
        if dir.join("bei.cfg.json").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    None
}

// Carregar configurações dos arquivos
pub fn load_configs() -> Result<BeiProps, Box<dyn std::error::Error>> {
    let root = find_project_root()
      .ok_or("Não foi possível encontrar o arquivo bei.cfg.json.\nExecute o comando dentro da raiz do projeto.")?;

    let cfg_path = root.join("bei.cfg.json");

    let cfg_content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Erro ao ler bei.cfg.json: {}", e))?;

    let config: BeiProps = serde_json::from_str(&cfg_content)?;

    Ok(config)
}

// Carregar credenciais do banco de dados a partir do backend/.env
pub fn load_db_config() -> Result<DbProps, Box<dyn std::error::Error>> {
    let root = find_project_root()
        .ok_or("Não foi possível encontrar o arquivo bei.cfg.json.\nExecute o comando dentro da raiz do projeto.")?;

    // Lê o backend_path do cfg principal
    let cfg_path = root.join("bei.cfg.json");
    let cfg_content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Erro ao ler bei.cfg.json: {}", e))?;
    let parsed: serde_json::Value = serde_json::from_str(&cfg_content)?;
    let backend_path = parsed["project_config"]["backend_path"]
        .as_str()
        .unwrap_or("backend");

    let env_path = root.join(backend_path).join(".env");

    if !env_path.exists() {
        return Err(format!(
            "Arquivo .env não encontrado em {}/. Execute 'bei init' primeiro.",
            backend_path
        )
        .into());
    }

    let env_content = std::fs::read_to_string(&env_path)
        .map_err(|e| format!("Erro ao ler o arquivo .env: {}", e))?;

    let mut user = String::new();
    let mut password = String::new();
    let mut database = String::new();

    for line in env_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "DB_USERNAME" => user = val.to_string(),
                "DB_PASSWORD" => password = val.to_string(),
                "DB_DATABASE" => database = val.to_string(),
                _ => {}
            }
        }
    }

    if user.is_empty() || database.is_empty() {
        return Err("O arquivo .env não possui as chaves DB_USERNAME e DB_DATABASE necessárias.".into());
    }

    Ok(DbProps {
        user,
        password,
        database,
    })
}
