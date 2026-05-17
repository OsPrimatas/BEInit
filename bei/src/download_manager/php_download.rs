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

    println!("PHP {} instalado com sucesso!", version);
    Ok(())
}
