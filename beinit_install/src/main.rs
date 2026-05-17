pub mod beinit_path;

use beinit_path::BEInitPaths;
use std::io::Cursor;
use winreg::RegKey;
use winreg::enums::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BEInit Setup - Iniciando instalação...");

    let paths = BEInitPaths::new();

    // 2. Baixar a ferramenta do GitHub
    let url = "https://github.com/OsPrimatas/BEInit/releases/latest/download/BEInit_Tool.zip";
    println!("Baixando a versão mais recente do BEInit...");

    let response = reqwest::blocking::get(url)?;
    let content = Cursor::new(response.bytes()?);

    println!(
        "Extraindo arquivos em: {}...",
        paths.get_beinit_path().display()
    );
    let mut archive = zip::ZipArchive::new(content)?;
    archive.extract(&paths.get_beinit_path())?;

    // 3. Configurar Variáveis de Ambiente
    println!("🔑 Configurando variáveis de ambiente...");
    setup_env_variables(paths.get_beinit_path().to_str().unwrap())?;

    // Aguarda o usuário ler antes de fechar
    println!("\nPressione Enter para sair...");
    let mut _input = String::new();
    std::io::stdin().read_line(&mut _input)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn setup_env_variables(path_value: &str) -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu.create_subkey("Environment")?;

    // Criar a variável BEINIT
    env.set_value("BEINIT", &path_value)?;
    println!("✅ Variável %BEINIT% criada.");

    // Inserir %BEINIT% no PATH (se não existir)
    let current_path: String = env.get_value("Path")?;
    if !current_path.contains("%BEINIT%") {
        let new_path = format!("{};%BEINIT%", current_path);
        env.set_value("Path", &new_path)?;
        println!("✅ %BEINIT% adicionado ao PATH do usuário.");
    }

    Ok(())
}
