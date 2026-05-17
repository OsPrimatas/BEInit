use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, Cursor};
use std::path::Path;
use tar::Archive;

pub async fn download_and_extract(
    url: &str,
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Iniciando download: {}", url);

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("Falha no download: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    println!(
        "Download concluído ({} MB)",
        bytes.len() as f64 / 1_048_576.0
    );

    // Cria o diretório de destino
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    println!("Extraindo arquivos para {:?}...", target_dir);

    // Detecta o formato pelo final da URL
    if url.contains(".zip") {
        extract_zip(&bytes, target_dir)?;
    } else if url.contains(".tar.gz") || url.contains(".tgz") {
        extract_tar_gz(&bytes, target_dir)?;
    } else if url.ends_with(".exe") || url.ends_with(".phar") {
        let filename = url.split('/').last().unwrap_or("download");
        let outpath = target_dir.join(filename);
        std::fs::write(outpath, &bytes)?;
    } else {
        return Err(format!("Formato de arquivo não suportado para a URL: {}", url).into());
    }

    println!("Extração concluída com sucesso!");
    Ok(())
}

// Extrair arquivos .zip
fn extract_zip(bytes: &[u8], target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

// Extrair arquivos .tar.gz
fn extract_tar_gz(bytes: &[u8], target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tar_gz = Cursor::new(bytes);
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(target_dir)?;
    Ok(())
}
