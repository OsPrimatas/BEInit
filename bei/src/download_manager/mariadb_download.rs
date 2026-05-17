use super::download_and_extract::download_and_extract;
use super::get_download_url::get_download_url;
use crate::utils::bei_paths::BeiPaths;

pub async fn download_mariadb_if_needed(
    version: &str,
    paths: &BeiPaths,
) -> Result<(), Box<dyn std::error::Error>> {
    let mariadb_dir = paths.ensure_version_dir("mariadb", version)?;

    // Verifica se o binário principal existe
    if paths.find_executable(&mariadb_dir, "mariadbd").is_some() {
        println!("MariaDB {} já está instalado.", version);
        return Ok(());
    }

    println!("Baixando MariaDB {}...", version);

    let url = get_download_url("mariadb", version)?;
    download_and_extract(&url, &mariadb_dir).await?;

    println!("MariaDB {} instalado com sucesso!", version);
    Ok(())
}
