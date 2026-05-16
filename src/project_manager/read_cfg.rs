use crate::utils::beinit_props::{BEInitDbProps, BEInitProps};
use std::env;
use std::path::PathBuf;

// Verificar se o arquivo beinit.cfg.json existe na raiz do projeto
pub fn find_project_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;

    loop {
        if dir.join("beinit.cfg.json").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    None
}

// Carregar configurações dos arquivos
pub fn load_configs() -> Result<(BEInitProps, BEInitDbProps), Box<dyn std::error::Error>> {
    let root = find_project_root()
      .ok_or("❌ Não foi possível encontrar o arquivo beinit.cfg.json.\nExecute o comando dentro da raiz do projeto.")?;

    let cfg_path = root.join("beinit.cfg.json");
    let db_path = root.join("beinit.db.json");

    let cfg_content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Erro ao ler beinit.cfg.json: {}", e))?;

    let db_content = std::fs::read_to_string(&db_path)
        .map_err(|e| format!("Erro ao ler beinit.db.json: {}", e))?;

    let config: BEInitProps = serde_json::from_str(&cfg_content)?;
    let db_config: BEInitDbProps = serde_json::from_str(&db_content)?;

    Ok((config, db_config))
}
