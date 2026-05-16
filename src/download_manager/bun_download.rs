use super::download_and_extract::download_and_extract;
use super::get_download_url::get_download_url;
use crate::utils::beinit_paths::BEInitPaths;

pub async fn download_bun_if_needed(paths: &BEInitPaths) -> Result<(), Box<dyn std::error::Error>> {
    let bun_dir = paths.ensure_version_dir("bun", "latest")?;

    // Verifica se o binário principal existe
    if paths.find_executable(&bun_dir, "bun").is_some() {
        println!("✅ Bun já está instalado.");
        return Ok(());
    }

    println!("⬇️  Baixando Bun (latest)...");

    let url = get_download_url("bun", "latest")?;
    download_and_extract(&url, &bun_dir).await?;

    println!("✅ Bun instalado com sucesso!");
    Ok(())
}
