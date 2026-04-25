use clap::Parser;
use colored::Colorize;

use graphdb_cli::cli::Cli;
use graphdb_cli::command::executor::CommandExecutor;
use graphdb_cli::command::parser::{self, HistoryAction, MetaCommand};
use graphdb_cli::command::script::is_statement_complete;
use graphdb_cli::config::settings::Config;
use graphdb_cli::input::handler::InputHandler;
use graphdb_cli::output::formatter::{OutputFormat, OutputFormatter};
use graphdb_cli::session::manager::SessionManager;
use graphdb_cli::utils::error::Result;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{}: {}", "ERROR".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    let host = if cli.host == "127.0.0.1" && config.connection.default_host != "127.0.0.1" {
        &config.connection.default_host
    } else {
        &cli.host
    };

    let port = if cli.port == 8080 && config.connection.default_port != 8080 {
        config.connection.default_port
    } else {
        cli.port
    };

    let user = if cli.user == "root" && config.connection.default_user != "root" {
        &config.connection.default_user
    } else {
        &cli.user
    };

    let password = if cli.password {
        read_password()?
    } else {
        String::new()
    };

    let mut session_mgr = SessionManager::new(host, port);

    let mut formatter = OutputFormatter::new();

    if let Some(fmt) = OutputFormat::parse(&cli.format) {
        formatter.set_format(fmt);
    } else if let Some(fmt) = OutputFormat::parse(&config.output.format) {
        formatter.set_format(fmt);
    }

    let mut executor = CommandExecutor::with_options(formatter, cli.force, cli.single_transaction);

    if !cli.variables.is_empty() {
        if let Some(session) = session_mgr.session_mut() {
            for var_def in &cli.variables {
                if let Some((name, value)) = var_def.split_once('=') {
                    session.set_variable(name.to_string(), value.to_string())?;
                }
            }
        }
    }

    match session_mgr.connect(user, &password).await {
        Ok(()) => {
            if !cli.quiet {
                println!("Connected to GraphDB at {}:{} as {}", host, port, user);
            }
        }
        Err(e) => {
            eprintln!("{}: Failed to connect: {}", "ERROR".red().bold(), e);
            eprintln!("Starting in offline mode. Use \\connect to connect to a server.");
        }
    }

    if let Some(ref space) = cli.database {
        match session_mgr.switch_space(space).await {
            Ok(()) => {
                if !cli.quiet {
                    println!("Space changed to '{}'", space);
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to switch to space '{}': {}",
                    "WARNING".yellow().bold(),
                    space,
                    e
                );
            }
        }
    }

    if !cli.variables.is_empty() {
        for var_def in &cli.variables {
            if let Some((name, value)) = var_def.split_once('=') {
                if let Some(session) = session_mgr.session_mut() {
                    session.set_variable(name.to_string(), value.to_string())?;
                }
            }
        }
    }

    if let Some(ref cmd) = cli.command {
        let command = parser::parse_command(cmd);
        match executor.execute(command, &mut session_mgr).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}: {}", "ERROR".red().bold(), e);
                if !cli.force {
                    std::process::exit(1);
                }
            }
        }
        return Ok(());
    }

    if let Some(ref file) = cli.file {
        let command =
            parser::Command::MetaCommand(parser::MetaCommand::ExecuteScript { path: file.clone() });
        match executor.execute(command, &mut session_mgr).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}: {}", "ERROR".red().bold(), e);
                if !cli.force {
                    std::process::exit(1);
                }
            }
        }
        return Ok(());
    }

    run_repl(&mut session_mgr, &mut executor).await
}

async fn run_repl(session_mgr: &mut SessionManager, executor: &mut CommandExecutor) -> Result<()> {
    let mut input_handler = InputHandler::new()?;

    loop {
        let prompt = session_mgr
            .session()
            .map(|s| s.prompt())
            .unwrap_or_else(|| "graphdb=# ".to_string());

        let line = match input_handler.read_line(&prompt)? {
            Some(line) => line,
            None => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('\\') {
            let command = parser::parse_command(trimmed);

            if let parser::Command::MetaCommand(MetaCommand::History { ref action }) = command {
                handle_history_repl(action, &mut input_handler, session_mgr, executor)?;
                continue;
            }

            match executor.execute(command, session_mgr).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "ERROR".red().bold(), e);
                }
            }

            let space = session_mgr.current_space();
            input_handler.add_history(trimmed, space);
            continue;
        }

        let mut full_input = line.clone();

        while !is_statement_complete(&full_input) {
            let cont_prompt = session_mgr
                .session()
                .map(|s| s.continuation_prompt())
                .unwrap_or_else(|| "graphdb-# ".to_string());

            match input_handler.read_line(&cont_prompt)? {
                Some(next_line) => {
                    full_input.push('\n');
                    full_input.push_str(&next_line);
                }
                None => break,
            }
        }

        let command = parser::parse_command(&full_input);
        match executor.execute(command, session_mgr).await {
            Ok(should_continue) => {
                if !should_continue {
                    break;
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "ERROR".red().bold(), e);
            }
        }

        let space = session_mgr.current_space();
        input_handler.add_history(&full_input, space);
    }

    input_handler.save_history();
    Ok(())
}

fn handle_history_repl(
    action: &HistoryAction,
    input_handler: &mut InputHandler,
    session_mgr: &mut SessionManager,
    _executor: &mut CommandExecutor,
) -> Result<()> {
    let mgr = input_handler.history_manager();
    match action {
        HistoryAction::Show { count } => {
            let entries = match count {
                Some(n) => mgr.recent(*n),
                None => mgr.recent(20),
            };
            if entries.is_empty() {
                println!("(history is empty)");
            } else {
                for entry in entries {
                    let space_info = entry
                        .space
                        .as_deref()
                        .map(|s| format!("[{}]", s))
                        .unwrap_or_default();
                    println!("  {} {}{}", entry.id, space_info, entry.command);
                }
            }
        }
        HistoryAction::Search { pattern } => {
            let results = mgr.search(pattern);
            if results.is_empty() {
                println!("(no matching history entries)");
            } else {
                for entry in results {
                    let space_info = entry
                        .space
                        .as_deref()
                        .map(|s| format!("[{}]", s))
                        .unwrap_or_default();
                    println!("  {} {}{}", entry.id, space_info, entry.command);
                }
            }
        }
        HistoryAction::Clear => {
            input_handler.history_manager_mut().clear()?;
            println!("History cleared.");
        }
        HistoryAction::Exec { id } => {
            if let Some(entry) = mgr.get_by_id(*id) {
                let cmd = entry.command.clone();
                let command = parser::parse_command(&cmd);
                let space = session_mgr.current_space();
                input_handler.add_history(&cmd, space);

                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    match _executor.execute(command, session_mgr).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("{}: {}", "ERROR".red().bold(), e);
                        }
                    }
                });
            } else {
                println!("History entry #{} not found.", id);
            }
        }
    }
    Ok(())
}

fn read_password() -> Result<String> {
    let password = rpassword::prompt_password("Password: ").map_err(|e| {
        graphdb_cli::utils::error::CliError::Other(format!("Failed to read password: {}", e))
    })?;
    Ok(password)
}
