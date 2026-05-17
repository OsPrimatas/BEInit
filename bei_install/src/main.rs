pub mod bei_path;

use bei_path::BeiPaths;
use std::io::Cursor;
use winreg::RegKey;
use winreg::enums::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Bei Setup - Iniciando instalação...");

    let paths = BeiPaths::new();

    // 2. Baixar a ferramenta do GitHub
    let url = "https://github.com/OsPrimatas/BEInit/releases/latest/download/Bei.zip";
    println!("Baixando a versão mais recente do Bei...");

    let response = reqwest::blocking::get(url)?;
    let content = Cursor::new(response.bytes()?);

    println!(
        "Extraindo arquivos em: {}...",
        paths.get_bei_path().display()
    );
    let mut archive = zip::ZipArchive::new(content)?;
    archive.extract(&paths.get_bei_path())?;

    // 3. Configurar Variáveis de Ambiente
    println!("Configurando variáveis de ambiente...");
    setup_env_variables(paths.get_bei_path().to_str().unwrap())?;

    // Aguarda o usuário ler antes de fechar
    println!("\nPressione Enter para sair...");
    let mut _input = String::new();
    std::io::stdin().read_line(&mut _input)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn setup_env_variables(path_value: &str) -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (env, _) =
        hkcu.create_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")?;

    // Criar a variável BEI
    env.set_value("BEI", &path_value)?;
    println!("Variável %BEI% criada.");

    // Inserir %BEI% no PATH (se não existir)
    let current_path: String = env.get_value("Path")?;
    if !current_path.contains("%BEI%") {
        let new_path = format!("{};%BEI%", current_path);
        env.set_value("Path", &new_path)?;
        println!("%BEI% adicionado ao PATH do usuário.");
    }

    Ok(())
}
