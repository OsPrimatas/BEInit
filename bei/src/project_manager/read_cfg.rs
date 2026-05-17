use crate::utils::bei_props::{BeiProps, DbConfig, DbProps};
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

// Carregar credenciais do banco de dados a partir do bei.db.json
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

    let db_json_path = root.join(backend_path).join("bei.db.json");

    if !db_json_path.exists() {
        return Err(format!(
            "Arquivo bei.db.json não encontrado em {}/. Execute 'bei init' primeiro.",
            backend_path
        )
        .into());
    }

    let db_content = std::fs::read_to_string(&db_json_path)
        .map_err(|e| format!("Erro ao ler bei.db.json: {}", e))?;

    let db_config: DbConfig = serde_json::from_str(&db_content)
        .map_err(|e| format!("Erro ao fazer parse do bei.db.json: {}", e))?;

    Ok(db_config.db)
}
