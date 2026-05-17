#![allow(unused)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectConfigProps {
    pub frontend_path: String,
    pub backend_path: String,
    pub add_gitignore: bool,
    pub add_env: bool,
    pub add_composer_file: bool,
}

#[derive(Debug, Deserialize)]
pub struct BunProps {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ComposerProps {
    pub version: String,
}

/// Configurações do bei
#[derive(Debug, Deserialize)]
pub struct BeiProps {
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

/// Credenciais do banco de dados (lidas do bei.db.json)
#[derive(Debug, Deserialize, Clone)]
pub struct DbProps {
    pub user: String,
    pub password: String,
    pub database: String,
}

/// Wrapper para o bei.db.json (campo raiz "db")
#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub db: DbProps,
}
