use std::env;

pub fn get_download_url(tool: &str, version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match tool {
        "php" => get_php_url(version, os),
        "mariadb" => get_mariadb_url(version, os),
        "bun" => get_bun_url(version, os, arch),
        _ => Err(format!("Ferramenta desconhecida: {}", tool).into()),
    }
}

fn get_php_url(version: &str, os: &str) -> Result<String, Box<dyn std::error::Error>> {
    match os {
        "windows" => Ok(format!(
            "https://windows.php.net/downloads/releases/archives/php-{}-nts-Win32-vs17-x64.zip",
            version
        )),
        "linux" => Err("Download automático de PHP no Linux não suportado. Use apt/dnf.".into()),
        _ => Err(format!("SO {} não suportado para PHP", os).into()),
    }
}

fn get_mariadb_url(version: &str, os: &str) -> Result<String, Box<dyn std::error::Error>> {
    match os {
        "windows" => Ok(format!(
            "https://archive.mariadb.org/mariadb-{version}/winx64-packages/mariadb-{version}-winx64.zip",
            version = version
        )),
        "linux" => Ok(format!(
            "https://archive.mariadb.org/mariadb-{version}/bintar-linux-systemd-x86_64/mariadb-{version}-linux-systemd-x86_64.tar.gz",
            version = version
        )),
        _ => Err(format!("SO {} não suportado para MariaDB", os).into()),
    }
}

fn get_bun_url(
    _version: &str,
    os: &str,
    _arch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Bun geralmente pegamos a latest
    match os {
        "windows" => Ok(
            "https://github.com/oven-sh/bun/releases/latest/download/bun-windows-x64.zip"
                .to_string(),
        ),
        "linux" => Ok(
            "https://github.com/oven-sh/bun/releases/latest/download/bun-linux-x64.tar.gz"
                .to_string(),
        ),
        _ => Err(format!("SO {} não suportado para Bun", os).into()),
    }
}
