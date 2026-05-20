use super::download_and_extract::download_and_extract;
use super::get_download_url::get_download_url;
use crate::utils::bei_paths::BeiPaths;

pub async fn download_composer_if_needed(
    version: &str,
    paths: &BeiPaths,
) -> Result<(), Box<dyn std::error::Error>> {
    let composer_dir = paths.ensure_version_dir("composer", version)?;

    // Se já estiver baixado, não faz nada.
    if composer_dir.join("composer.phar").exists() {
        println!("Composer {} já está instalado.", version);
        return Ok(());
    }

    println!("Baixando Composer {}...", version);

    // URL do composer.
    let url = get_download_url("composer", version)?;

    // Baixar + salvar como composer.phar.
    download_and_extract(&url, &composer_dir).await?;

    // Create a .bat wrapper for Windows
    if cfg!(target_os = "windows") {
        let bat_content = "@ECHO OFF\nphp \"%~dp0composer.phar\" %*\n";
        std::fs::write(composer_dir.join("composer.bat"), bat_content)?;
    } else {
        let sh_content = "#!/bin/sh\nphp \"$(dirname \"$0\")/composer.phar\" \"$@\"\n";
        std::fs::write(composer_dir.join("composer"), sh_content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(composer_dir.join("composer"))?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(composer_dir.join("composer"), perms)?;
        }
    }

    println!("Composer {} instalado com sucesso!", version);
    Ok(())
}
