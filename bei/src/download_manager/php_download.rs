use super::download_and_extract::download_and_extract;
use super::get_download_url::get_download_url;
use crate::utils::bei_paths::BeiPaths;

pub async fn download_php_if_needed(
    version: &str,
    paths: &BeiPaths,
) -> Result<(), Box<dyn std::error::Error>> {
    let php_dir = paths.ensure_version_dir("php", version)?;

    // Se já estiver baixado, não faz nada.
    if paths.find_executable(&php_dir, "php").is_some() {
        println!("PHP {} já está instalado.", version);
        return Ok(());
    }

    println!("Baixando PHP {}...", version);

    // URL dinâmica dependendo do SO.
    let url = get_download_url("php", version)?;

    // Baixar + extrair.
    download_and_extract(&url, &php_dir).await?;

    // Configurar php.ini
    if let Err(e) = configure_php_ini(&php_dir) {
        println!("Aviso: Falha ao configurar o php.ini: {}", e);
    }

    println!("PHP {} instalado com sucesso!", version);
    Ok(())
}

fn configure_php_ini(php_dir: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let dev_ini = php_dir.join("php.ini-development");
    let target_ini = php_dir.join("php.ini");

    if dev_ini.exists() {
        let mut content = std::fs::read_to_string(&dev_ini)?;

        // No Windows, a pasta de extensões é ext
        content = content.replace(";extension_dir = \"ext\"", "extension_dir = \"ext\"");
        
        // Habilitar extensões de banco de dados
        content = content.replace(";extension=pdo_mysql", "extension=pdo_mysql");
        content = content.replace(";extension=mysqli", "extension=mysqli");
        content = content.replace(";extension=mbstring", "extension=mbstring");
        content = content.replace(";extension=curl", "extension=curl");
        content = content.replace(";extension=openssl", "extension=openssl");

        std::fs::write(&target_ini, content)?;
    }
    
    Ok(())
}
