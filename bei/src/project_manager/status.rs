use crate::project_manager::read_cfg::find_project_root;
use crate::utils::bei_paths::BeiPaths;
use colored::Colorize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

async fn is_port_open(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    if let Ok(socket_addr) = addr.parse::<std::net::SocketAddr>() {
        match timeout(
            Duration::from_millis(300),
            TcpStream::connect(&socket_addr),
        )
        .await
        {
            Ok(Ok(_)) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn is_pid_running(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(status) = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
        {
            status.success()
        } else {
            false
        }
    }
}

#[derive(Debug)]
enum ServiceState {
    ActiveBei(u32),
    ActiveConflict,
    Inactive,
    Crashed(u32),
}

async fn analyze_service_state(port: u16, saved_pid: Option<u32>) -> ServiceState {
    let port_open = is_port_open(port).await;
    match (port_open, saved_pid) {
        (true, Some(pid)) => {
            if is_pid_running(pid) {
                ServiceState::ActiveBei(pid)
            } else {
                ServiceState::ActiveConflict
            }
        }
        (true, None) => ServiceState::ActiveConflict,
        (false, Some(pid)) => {
            ServiceState::Crashed(pid)
        }
        (false, None) => ServiceState::Inactive,
    }
}

pub async fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "======================================================".cyan().bold());
    println!("{}", "               STATUS DO AMBIENTE BEI                 ".cyan().bold());
    println!("{}", "======================================================".cyan().bold());
    println!();

    // 1. Verificar se o projeto está inicializado
    let root = match find_project_root() {
        Some(r) => r,
        None => {
            println!("{}", "Nenhum projeto bei detectado nesta pasta.".red().bold());
            println!("Para inicializar um novo projeto, execute: {}", "bei init".yellow());
            return Ok(());
        }
    };

    // Tentar carregar bei.cfg.json manualmente para validação granular
    let cfg_json_path = root.join("bei.cfg.json");
    let mut cfg_ok = false;
    let mut cfg_err_msg = String::new();
    let mut config_opt = None;

    if cfg_json_path.exists() {
        match std::fs::read_to_string(&cfg_json_path) {
            Ok(content) => {
                match serde_json::from_str::<crate::utils::bei_props::BeiProps>(&content) {
                    Ok(cfg) => {
                        config_opt = Some(cfg);
                        cfg_ok = true;
                    }
                    Err(e) => {
                        cfg_err_msg = format!("Erro ao fazer parse do JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                cfg_err_msg = format!("Erro ao ler o arquivo: {}", e);
            }
        }
    } else {
        cfg_err_msg = "Arquivo bei.cfg.json não foi encontrado na raiz do projeto.".to_string();
    }

    println!("{}", "--- CONFIGURAÇÃO E ESTRUTURA ---".blue().bold());

    if !cfg_ok {
        println!("{:<45} [{}]", "Configuração Principal (bei.cfg.json)", "ERRO DE FORMATO".red().bold());
        if !cfg_err_msg.is_empty() {
            println!("   {} {}", "Erro Técnico:".red().bold(), cfg_err_msg.red());
        }
        println!("   {} {}", "Explicação:".yellow(), "O arquivo de configuração principal está corrompido ou contém sintaxe JSON inválida.".red());
        println!("   {} {}", "Como resolver:".green(), "Abra o arquivo bei.cfg.json na raiz do projeto e corrija a sintaxe (verifique vírgulas duplicadas, aspas ausentes ou chaves incorretas). Se preferir, recrie-o rodando 'bei init'.".yellow());
        println!();
        println!("{}", "======================================================".cyan().bold());
        return Ok(());
    }

    let config = config_opt.unwrap();

    // 2. Pasta Frontend
    let frontend_path = root.join(&config.project_config.frontend_path);
    if !frontend_path.exists() {
        print_file_status(
            &format!("Pasta Frontend ({})", config.project_config.frontend_path),
            "AUSENTE",
            "",
            &format!("A pasta do frontend '{}' não existe na raiz do projeto.", config.project_config.frontend_path),
            &format!("Crie a pasta executando 'mkdir {}' ou revise a configuração 'frontend_path' no seu bei.cfg.json.", config.project_config.frontend_path)
        );
    } else {
        print_file_status(
            &format!("Pasta Frontend ({})", config.project_config.frontend_path),
            "OK",
            "",
            "",
            ""
        );
    }

    // 3. Pasta Backend
    let backend_path = root.join(&config.project_config.backend_path);
    if !backend_path.exists() {
        print_file_status(
            &format!("Pasta Backend ({})", config.project_config.backend_path),
            "AUSENTE",
            "",
            &format!("A pasta do backend '{}' não existe na raiz do projeto.", config.project_config.backend_path),
            &format!("Crie a pasta executando 'mkdir {}' ou revise a configuração 'backend_path' no seu bei.cfg.json.", config.project_config.backend_path)
        );
    } else {
        print_file_status(
            &format!("Pasta Backend ({})", config.project_config.backend_path),
            "OK",
            "",
            "",
            ""
        );
    }

    // 4. Arquivo .gitignore
    let gitignore_path = root.join(".gitignore");
    if !gitignore_path.exists() {
        print_file_status(
            "Arquivo .gitignore",
            "AUSENTE",
            "",
            "O arquivo .gitignore não foi encontrado na raiz do projeto.",
            "É altamente recomendável criar um .gitignore para evitar o envio acidental da pasta de dados local '.bei/' para o Git (o que pode expor senhas do banco de dados e arquivos pesados)."
        );
    } else {
        print_file_status(
            "Arquivo .gitignore",
            "OK",
            "",
            "",
            ""
        );
    }

    // 5. Arquivo de Ambiente (.env)
    let env_path = backend_path.join(".env");
    if !env_path.exists() {
        print_file_status(
            "Arquivo de Ambiente (backend/.env)",
            "AUSENTE",
            "",
            "O arquivo .env não existe na pasta do backend.",
            "Ele é utilizado para configurar chaves secretas e configurações de ambiente de forma segura. Crie o arquivo ou execute 'bei init' novamente."
        );
    } else {
        print_file_status(
            "Arquivo de Ambiente (backend/.env)",
            "OK",
            "",
            "",
            ""
        );
    }

    // 6. Arquivo bei.db.json com validação de parsing JSON
    let db_json_path = backend_path.join("bei.db.json");
    let mut db_ok = false;
    let mut db_err_msg = String::new();
    let mut db_config_opt = None;

    if db_json_path.exists() {
        match std::fs::read_to_string(&db_json_path) {
            Ok(content) => {
                match serde_json::from_str::<crate::utils::bei_props::DbConfig>(&content) {
                    Ok(dbc) => {
                        db_config_opt = Some(dbc.db);
                        db_ok = true;
                    }
                    Err(e) => {
                        db_err_msg = format!("Erro ao fazer parse do JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                db_err_msg = format!("Erro ao ler o arquivo: {}", e);
            }
        }
    } else {
        db_err_msg = "Arquivo bei.db.json não foi encontrado.".to_string();
    }

    let db_exists = db_json_path.exists();
    if !db_exists {
        print_file_status(
            "Configuração do Banco (backend/bei.db.json)",
            "AUSENTE",
            &db_err_msg,
            "O arquivo backend/bei.db.json não existe. O MariaDB necessita dele para saber a senha, nome do banco e usuário a serem inicializados.",
            "Execute 'bei init' para criar o arquivo com os valores padrão de banco de dados."
        );
    } else if !db_ok {
        print_file_status(
            "Configuração do Banco (backend/bei.db.json)",
            "ERRO DE FORMATO",
            &db_err_msg,
            "O arquivo backend/bei.db.json está corrompido ou com erros de sintaxe JSON.",
            "Abra o arquivo backend/bei.db.json e corrija a formatação do JSON (verifique se há vírgulas sobrando ou chaves ausentes)."
        );
    } else {
        print_file_status(
            "Configuração do Banco (backend/bei.db.json)",
            "OK",
            "",
            "",
            ""
        );
    }

    println!();

    // --- STATUS DAS FERRAMENTAS INSTALADAS ---
    println!("{}", "--- FERRAMENTAS INSTALADAS ---".blue().bold());
    let paths = BeiPaths::new();

    // PHP
    let php_dir = paths.get_php_dir(&config.php.version);
    let php_exe = paths.find_executable(&php_dir, "php");
    let (php_state, php_explanation, php_suggestion) = if php_exe.is_some() {
        ("Instalado", "".to_string(), "".to_string())
    } else if php_dir.exists() {
        (
            "Corrompido",
            format!("A pasta da versão configurada (v{}) existe em '{}', mas o executável 'php' correspondente não foi encontrado.", config.php.version, php_dir.display()),
            "A instalação pode ter sido corrompida. Delete a pasta correspondente e execute 'bei install' para reinstalar de forma limpa.".to_string()
        )
    } else {
        (
            "Não Instalado",
            format!("A pasta da versão configurada (v{}) não existe em '.bei/bin/php/'.", config.php.version),
            "Execute o comando 'bei install' para baixar e instalar o PHP de forma automatizada.".to_string()
        )
    };
    print_tool_status("PHP", &config.php.version, php_state, php_exe, &php_explanation, &php_suggestion);

    // MariaDB
    let mariadb_dir = paths.get_mariadb_dir(&config.mariadb.version);
    let mariadb_exe = paths.find_executable(&mariadb_dir, "mariadbd")
        .or_else(|| paths.find_executable(&mariadb_dir, "mysqld"));
    let (mariadb_state, mariadb_explanation, mariadb_suggestion) = if mariadb_exe.is_some() {
        ("Instalado", "".to_string(), "".to_string())
    } else if mariadb_dir.exists() {
        (
            "Corrompido",
            format!("A pasta da versão configurada (v{}) existe, mas os executáveis do servidor ('mariadbd' ou 'mysqld') estão ausentes.", config.mariadb.version),
            "A instalação do banco de dados está incompleta. Exclua a pasta '.bei/bin/mariadb/' e execute 'bei install'.".to_string()
        )
    } else {
        (
            "Não Instalado",
            format!("A pasta da versão configurada (v{}) do MariaDB não foi encontrada em '.bei/bin/mariadb/'.", config.mariadb.version),
            "O banco de dados local não está instalado. Execute o comando 'bei install' para baixá-lo e configurá-lo.".to_string()
        )
    };
    print_tool_status("MariaDB", &config.mariadb.version, mariadb_state, mariadb_exe, &mariadb_explanation, &mariadb_suggestion);

    // Bun
    let bun_dir = paths.get_bun_dir("latest");
    let bun_exe = paths.find_executable(&bun_dir, "bun");
    let (bun_state, bun_explanation, bun_suggestion) = if bun_exe.is_some() {
        ("Instalado", "".to_string(), "".to_string())
    } else if bun_dir.exists() {
        (
            "Corrompido",
            format!("A pasta do Bun existe em '{}', mas o executável 'bun' não foi encontrado.", bun_dir.display()),
            "A reinstalação pode ser feita excluindo a pasta '.bei/bin/bun/' e executando 'bei install'.".to_string()
        )
    } else {
        (
            "Não Instalado",
            "O executável do Bun não foi encontrado no seu diretório '.bei/bin/bun/'.".to_string(),
            "O Bun gerencia dependências e executa o servidor do frontend em altíssima velocidade. Execute 'bei install' para instalá-lo.".to_string()
        )
    };
    print_tool_status("Bun", &config.bun.version, bun_state, bun_exe, &bun_explanation, &bun_suggestion);

    // Composer
    let composer_dir = paths.bin_dir.join("composer").join(&config.composer.version);
    let composer_exe = paths.find_executable(&composer_dir, "composer");
    let (composer_state, composer_explanation, composer_suggestion) = if composer_exe.is_some() {
        ("Instalado", "".to_string(), "".to_string())
    } else if composer_dir.exists() {
        (
            "Corrompido",
            format!("A pasta do Composer existe em '{}', mas o executável 'composer' correspondente não foi encontrado.", composer_dir.display()),
            "Exclua a pasta do Composer em '.bei/bin/composer/' e execute 'bei install' para reinstalar.".to_string()
        )
    } else {
        (
            "Não Instalado",
            format!("A pasta do Composer (v{}) não existe em '.bei/bin/composer/'.", config.composer.version),
            "O gerenciador de dependências PHP Composer está ausente. Execute 'bei install' para realizar o download automático.".to_string()
        )
    };
    print_tool_status("Composer", &config.composer.version, composer_state, composer_exe, &composer_explanation, &composer_suggestion);

    println!();

    // --- STATUS DE EXECUÇÃO DOS SERVIÇOS ---
    println!("{}", "--- SERVIÇOS EM EXECUÇÃO ---".blue().bold());

    // Carregar PIDs salvos em .bei/pids.json se existirem
    let pids_file = root.join(".bei").join("pids.json");
    let mut saved_pids = serde_json::Value::Null;
    if pids_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&pids_file) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                saved_pids = parsed;
            }
        }
    }

    // PHP Dev Server
    let php_pid = saved_pids["php"].as_u64().map(|p| p as u32);
    let php_state = analyze_service_state(config.php.port, php_pid).await;
    print_service_state_details(
        "PHP Dev Server",
        config.php.port,
        &php_state,
        &format!("http://localhost:{}", config.php.port)
    );

    // MariaDB
    let db_name = db_config_opt.as_ref().map(|d| d.database.as_str()).unwrap_or("bei_db");
    let mariadb_pid = saved_pids["mariadb"].as_u64().map(|p| p as u32);
    let mariadb_state = analyze_service_state(config.mariadb.port, mariadb_pid).await;
    print_service_state_details(
        "MariaDB Server",
        config.mariadb.port,
        &mariadb_state,
        &format!("localhost:{} (Banco: {})", config.mariadb.port, db_name)
    );

    // Frontend (Vite/Bun) - Encontrar se alguma das portas 5173..=5175 está aberta
    let bun_pid = saved_pids["bun"].as_u64().map(|p| p as u32);
    let mut open_frontend_port = None;
    for port in 5173..=5175 {
        if is_port_open(port).await {
            open_frontend_port = Some(port);
            break;
        }
    }

    let frontend_port_used = open_frontend_port.unwrap_or(5173);
    let frontend_state = match (open_frontend_port.is_some(), bun_pid) {
        (true, Some(pid)) => {
            if is_pid_running(pid) {
                ServiceState::ActiveBei(pid)
            } else {
                ServiceState::ActiveConflict
            }
        }
        (true, None) => ServiceState::ActiveConflict,
        (false, Some(pid)) => ServiceState::Crashed(pid),
        (false, None) => ServiceState::Inactive,
    };

    print_service_state_details(
        "Frontend (Vite/Bun)",
        frontend_port_used,
        &frontend_state,
        &format!("http://localhost:{}", frontend_port_used)
    );

    println!();
    println!("{}", "======================================================".cyan().bold());
    Ok(())
}

fn print_file_status(
    label: &str,
    state: &str,
    err_details: &str,
    explanation: &str,
    suggestion: &str,
) {
    let state_colored = match state {
        "OK" => "OK".green().bold(),
        "AUSENTE" => "AUSENTE".red().bold(),
        "ERRO DE FORMATO" => "ERRO DE FORMATO".red().bold(),
        _ => state.red().bold(),
    };

    println!("{:<45} [{}]", label, state_colored);
    if state != "OK" {
        if !err_details.is_empty() {
            println!("   {} {}", "Erro Técnico:".red().bold(), err_details.red());
        }
        println!("   {} {}", "Explicação:".yellow(), explanation.red());
        if !suggestion.is_empty() {
            println!("   {} {}", "Como resolver:".green(), suggestion.yellow());
        }
    }
}

fn print_tool_status(
    name: &str,
    version: &str,
    state: &str,
    path: Option<std::path::PathBuf>,
    explanation: &str,
    suggestion: &str,
) {
    let state_colored = match state {
        "Instalado" => "Instalado".green().bold(),
        "Não Instalado" => "Não Instalado".red().bold(),
        "Corrompido" => "Corrompido".red().bold(),
        _ => state.red().bold(),
    };

    println!("{:<12} (v{:<8}) -> {}", name, version, state_colored);
    if state == "Instalado" {
        if let Some(p) = path {
            println!("   Caminho: {}", p.display().to_string().dimmed());
        }
    } else {
        println!("   {} {}", "Explicação:".yellow(), explanation.red());
        println!("   {} {}", "Como resolver:".green(), suggestion.yellow());
    }
}

fn print_service_state_details(
    name: &str,
    port: u16,
    state: &ServiceState,
    info: &str,
) {
    match state {
        ServiceState::ActiveBei(pid) => {
            println!("{:<22} (Porta {:<5}) -> {} (PID: {}, {})", 
                name, 
                port, 
                "ATIVO".green().bold(), 
                pid, 
                info.dimmed()
            );
        }
        ServiceState::ActiveConflict => {
            println!("{:<22} (Porta {:<5}) -> {} ({})", 
                name, 
                port, 
                "CONFLITO DE PORTA".red().bold(), 
                info.dimmed()
            );
            println!("   {} {}", "Explicação:".yellow(), format!("A porta {} está aberta, mas o serviço não foi iniciado por este projeto bei.", port).red());
            println!("   {} {}", "Causa provável:".yellow(), "Existe outra aplicação em execução ocupando essa porta (ex: Docker, XAMPP, outra instância local ou IIS).".red());
            println!("   {} {}", "Como resolver:".green(), format!("Finalize o processo que está ocupando a porta {} ou mude a porta configurada no seu arquivo bei.cfg.json.", port).yellow());
        }
        ServiceState::Inactive => {
            println!("{:<22} (Porta {:<5}) -> {}", 
                name, 
                port, 
                "INATIVO".red().bold()
            );
            println!("   {} {}", "Explicação:".yellow(), "O serviço está totalmente desligado.".red());
            println!("   {} {}", "Como resolver:".green(), "Inicie todo o ambiente de desenvolvimento executando o comando 'bei run'.".yellow());
        }
        ServiceState::Crashed(pid) => {
            println!("{:<22} (Porta {:<5}) -> {}", 
                name, 
                port, 
                "TRAVADO / INATIVO INESPERADAMENTE".red().bold()
            );
            println!("   {} {}", "Explicação:".yellow(), format!("O serviço está desligado na porta {}, mas existe um registro de PID ativo anterior (PID: {}).", port, pid).red());
            println!("   {} {}", "Causa provável:".yellow(), "O processo foi finalizado de maneira forçada pelo gerenciador de tarefas ou sofreu um crash interno.".red());
            println!("   {} {}", "Como resolver:".green(), "Execute o comando 'bei stop' para realizar uma limpeza completa no estado do ambiente e então use 'bei run' novamente.".yellow());
        }
    }
}
