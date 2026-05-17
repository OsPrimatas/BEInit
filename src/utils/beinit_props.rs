#![allow(unused)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectConfigProps {
    pub project_name: String,
    pub frontend_path: String,
    pub backend_path: String,
    pub add_gitignore: bool,
    pub add_env: bool,
    pub add_composer_file: bool,
    pub add_frontend_folder: bool,
    pub add_backend_folder: bool,
}

#[derive(Debug, Deserialize)]
pub struct BunProps {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ComposerProps {
    pub version: String,
}

/// Configurações do BEInit
#[derive(Debug, Deserialize)]
pub struct BEInitProps {
    pub project_config: ProjectConfigProps,
    pub bun: BunProps,
    pub php: PhpProps,
    pub mariadb: MariaDbProps,
    pub composer: ComposerProps,
}

/// Configurações do PHP
#[derive(Debug, Deserialize)]
pub struct PhpProps {
    pub version: String,
    pub port: u16,
    pub display_errors: bool,
}

/// Configurações do MariaDB
#[derive(Debug, Deserialize)]
pub struct MariaDbProps {
    pub version: String,
    pub port: u16,
    pub data_dir: String,
}

