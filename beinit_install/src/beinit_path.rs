use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BEInitPaths {
    pub base_dir: PathBuf,
    pub bin_dir: PathBuf,
}

impl BEInitPaths {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("com", "os_primatas", "beinit")
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

    /// Retorna o caminho: bin/beinit/{version}/
    pub fn get_beinit_tool_path(&self) -> PathBuf {
        self.bin_dir.join("beinit_tool")
    }

    /// Procura recursivamente por um executável com o nome dado dentro de um diretório
    pub fn find_executable(&self, dir: &Path, name: &str) -> Option<PathBuf> {
        if !dir.exists() {
            return None;
        }
        let exe_name = format!("{}{}", name, std::env::consts::EXE_SUFFIX);
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            if let Ok(entries) = fs::read_dir(current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            if file_name == exe_name {
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
