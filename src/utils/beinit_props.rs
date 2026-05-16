#![allow(unused)]
use serde::Deserialize;

/// Configurações do BEInit
#[derive(Debug, Deserialize)]
pub struct BEInitProps {
    pub frontend_path: String,
    pub backend_path: String,
    pub php: PhpProps,
    pub mariadb: MariaDbProps,
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

/// Configurações do BEInit DB
#[derive(Debug, Deserialize)]
pub struct BEInitDbProps {
    pub mariadb_user: String,
    pub mariadb_password: String,
    pub mariadb_database: String,
}
