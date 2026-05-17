use crate::utils::beinit_props::BEInitProps;
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
pub fn load_configs() -> Result<BEInitProps, Box<dyn std::error::Error>> {
    let root = find_project_root()
      .ok_or("❌ Não foi possível encontrar o arquivo beinit.cfg.json.\nExecute o comando dentro da raiz do projeto.")?;

    let cfg_path = root.join("beinit.cfg.json");

    let cfg_content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Erro ao ler beinit.cfg.json: {}", e))?;

    let config: BEInitProps = serde_json::from_str(&cfg_content)?;

    Ok(config)
}
