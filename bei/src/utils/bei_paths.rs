use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BeiPaths {
    pub base_dir: PathBuf,
    pub bin_dir: PathBuf,
}

impl BeiPaths {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("com", "os_primatas", "bei")
            .expect("Não foi possível determinar as pastas de sistema.");

        let base_dir = proj_dirs.data_dir().to_path_buf();
        let bin_dir = base_dir.join("bin");

        Self { base_dir, bin_dir }
    }

    pub fn ensure_dirs_exist(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        fs::create_dir_all(&self.bin_dir)?;
        Ok(())
    }

    /// Retorna o caminho: bin/php/{version}/
    pub fn get_php_dir(&self, version: &str) -> PathBuf {
        self.bin_dir.join("php").join(version)
    }

    /// Retorna o caminho: bin/mariadb/{version}/
    pub fn get_mariadb_dir(&self, version: &str) -> PathBuf {
        self.bin_dir.join("mariadb").join(version)
    }

    /// Retorna o caminho: bin/bun/{version}/
    pub fn get_bun_dir(&self, version: &str) -> PathBuf {
        self.bin_dir.join("bun").join(version)
    }

    /// Cria a pasta da versão se não existir
    pub fn ensure_version_dir(&self, tool: &str, version: &str) -> std::io::Result<PathBuf> {
        let dir = match tool {
            "php" => self.get_php_dir(version),
            "mariadb" => self.get_mariadb_dir(version),
            "bun" => self.get_bun_dir(version),
            _ => self.bin_dir.join(tool).join(version),
        };

        if !dir.exists() {
            fs::create_dir_all(&dir)?;
            println!("📁 Pasta criada: {:?}", dir);
        }
        Ok(dir)
    }

    /// Procura recursivamente por um executável com o nome dado dentro de um diretório
    pub fn find_executable(&self, dir: &Path, name: &str) -> Option<PathBuf> {
        if !dir.exists() {
            return None;
        }
        
        let valid_names = if cfg!(target_os = "windows") {
            vec![
                format!("{}.exe", name),
                format!("{}.bat", name),
                format!("{}.cmd", name),
                name.to_string(),
            ]
        } else {
            vec![name.to_string()]
        };

        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            if let Ok(entries) = fs::read_dir(current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            if valid_names.iter().any(|n| n == file_name) {
                                return Some(path);
                            }
                        }
                    } else if path.is_dir() {
                        stack.push(path);
                    }
                }
            }
        }
        None
    }
}
