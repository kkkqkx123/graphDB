use std::fs;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;

use crate::command::meta_commands;
use crate::command::parser::{Command, MetaCommand};
use crate::output::formatter::OutputFormatter;
use crate::output::pager::Pager;
use crate::session::manager::SessionManager;
use crate::utils::error::{CliError, Result};

pub struct CommandExecutor {
    formatter: OutputFormatter,
    pager: Pager,
    output_file: Option<std::fs::File>,
}

impl CommandExecutor {
    pub fn new(formatter: OutputFormatter) -> Self {
        Self {
            formatter,
            pager: Pager::new(),
            output_file: None,
        }
    }

    pub fn formatter(&self) -> &OutputFormatter {
        &self.formatter
    }

    pub fn formatter_mut(&mut self) -> &mut OutputFormatter {
        &mut self.formatter
    }

    pub fn execute<'a>(
        &'a mut self,
        command: Command,
        session_mgr: &'a mut SessionManager,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async move {
            match command {
                Command::Empty => Ok(true),
                Command::Query(query) => self.execute_query(&query, session_mgr).await,
                Command::MetaCommand(meta) => self.execute_meta(meta, session_mgr).await,
            }
        })
    }

    async fn execute_query(
        &mut self,
        query: &str,
        session_mgr: &mut SessionManager,
    ) -> Result<bool> {
        let query = query.trim();

        if query.is_empty() {
            return Ok(true);
        }

        let use_match = query.trim_start().to_uppercase();
        if use_match.starts_with("USE ") {
            let space = query.trim_start()[4..].trim().trim_end_matches(';').trim();
            return self.execute_use_space(space, session_mgr).await;
        }

        let result = session_mgr.execute_query(query).await?;

        let output = self.formatter.format_result(&result);
        self.write_output(&output)?;

        Ok(true)
    }

    async fn execute_use_space(
        &mut self,
        space: &str,
        session_mgr: &mut SessionManager,
    ) -> Result<bool> {
        session_mgr.switch_space(space).await?;
        self.write_output(&format!("Space changed to '{}'", space))?;
        Ok(true)
    }

    async fn execute_meta(
        &mut self,
        meta: MetaCommand,
        session_mgr: &mut SessionManager,
    ) -> Result<bool> {
        match meta {
            MetaCommand::Quit => {
                self.write_output("Goodbye!")?;
                Ok(false)
            }
            MetaCommand::Help { topic } => {
                let help = meta_commands::show_help(topic.as_deref());
                self.write_output(&help)?;
                Ok(true)
            }
            MetaCommand::Connect { space } => {
                session_mgr.switch_space(&space).await?;
                self.write_output(&format!("Connected to space '{}'", space))?;
                Ok(true)
            }
            MetaCommand::Disconnect => {
                session_mgr.disconnect()?;
                self.write_output("Disconnected.")?;
                Ok(true)
            }
            MetaCommand::ConnInfo => {
                let info = session_mgr
                    .session()
                    .map(|s| s.conninfo())
                    .unwrap_or_else(|| "Not connected".to_string());
                self.write_output(&info)?;
                Ok(true)
            }
            MetaCommand::ShowSpaces => {
                let spaces = session_mgr.list_spaces().await?;
                let output = self.formatter.format_spaces(&spaces);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::ShowTags { .. } => {
                let tags = session_mgr.list_tags().await?;
                let output = self.formatter.format_tags(&tags);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::ShowEdges { .. } => {
                let edge_types = session_mgr.list_edge_types().await?;
                let output = self.formatter.format_edge_types(&edge_types);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::ShowIndexes { .. } => {
                self.write_output("Index listing is not yet supported via CLI.")?;
                Ok(true)
            }
            MetaCommand::ShowUsers => {
                self.write_output("User listing is not yet supported via CLI.")?;
                Ok(true)
            }
            MetaCommand::ShowFunctions => {
                self.write_output("Function listing is not yet supported via CLI.")?;
                Ok(true)
            }
            MetaCommand::Describe { object } => {
                let tags = session_mgr.list_tags().await?;
                if let Some(tag) = tags.iter().find(|t| t.name == object) {
                    let output = self.formatter.format_describe_tag(tag);
                    self.write_output(&output)?;
                } else {
                    self.write_output(
                        &self
                            .formatter
                            .format_error(&format!("Tag '{}' not found", object)),
                    )?;
                }
                Ok(true)
            }
            MetaCommand::DescribeEdge { name } => {
                let edge_types = session_mgr.list_edge_types().await?;
                if let Some(et) = edge_types.iter().find(|e| e.name == name) {
                    let output = self.formatter.format_describe_edge(et);
                    self.write_output(&output)?;
                } else {
                    self.write_output(
                        &self
                            .formatter
                            .format_error(&format!("Edge type '{}' not found", name)),
                    )?;
                }
                Ok(true)
            }
            MetaCommand::Format { format } => {
                self.formatter.set_format(format);
                self.write_output(&format!(
                    "Output format set to: {}",
                    self.formatter.format().as_str()
                ))?;
                Ok(true)
            }
            MetaCommand::Pager { command } => {
                self.pager.set_command(command);
                match self.pager.command() {
                    Some(cmd) => {
                        self.write_output(&format!("Pager set to: {}", cmd))?;
                    }
                    None => {
                        self.write_output("Pager disabled.")?;
                    }
                }
                Ok(true)
            }
            MetaCommand::Timing => {
                let current = self.formatter.timing_enabled();
                self.formatter.set_timing(!current);
                self.write_output(&format!(
                    "Timing {}.",
                    if !current { "enabled" } else { "disabled" }
                ))?;
                Ok(true)
            }
            MetaCommand::Set { name, value } => {
                let session = session_mgr.session_mut().ok_or(CliError::NotConnected)?;
                match value {
                    Some(v) => {
                        session.set_variable(name.clone(), v);
                        self.write_output(&format!(
                            "Set variable: {} = {}",
                            name,
                            session.get_variable(&name).unwrap()
                        ))?;
                    }
                    None => {
                        if let Some(v) = session.get_variable(&name) {
                            self.write_output(&format!("{} = {}", name, v))?;
                        } else {
                            self.write_output(&format!("Variable '{}' is not set", name))?;
                        }
                    }
                }
                Ok(true)
            }
            MetaCommand::Unset { name } => {
                let session = session_mgr.session_mut().ok_or(CliError::NotConnected)?;
                session.remove_variable(&name);
                self.write_output(&format!("Unset variable: {}", name))?;
                Ok(true)
            }
            MetaCommand::ShowVariables => {
                let session = session_mgr.session().ok_or(CliError::NotConnected)?;
                if session.variables.is_empty() {
                    self.write_output("(no variables set)")?;
                } else {
                    let mut output = String::new();
                    for (name, value) in &session.variables {
                        output.push_str(&format!("{} = {}\n", name, value));
                    }
                    self.write_output(output.trim_end())?;
                }
                Ok(true)
            }
            MetaCommand::ExecuteScript { path } => {
                self.execute_script(&path, session_mgr).await?;
                Ok(true)
            }
            MetaCommand::OutputRedirect { path } => {
                match path {
                    Some(p) => {
                        let file = std::fs::File::create(&p).map_err(CliError::IoError)?;
                        self.output_file = Some(file);
                        self.write_output(&format!("Output redirected to: {}", p))?;
                    }
                    None => {
                        self.output_file = None;
                        self.write_output("Output redirect closed.")?;
                    }
                }
                Ok(true)
            }
            MetaCommand::ShellCommand { command } => {
                #[cfg(target_os = "windows")]
                let result = std::process::Command::new("cmd")
                    .args(["/C", &command])
                    .output();

                #[cfg(not(target_os = "windows"))]
                let result = std::process::Command::new("sh")
                    .args(["-c", &command])
                    .output();

                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if !stdout.is_empty() {
                            self.write_output(&stdout)?;
                        }
                        if !stderr.is_empty() {
                            self.write_output(&self.formatter.format_error(&stderr))?;
                        }
                    }
                    Err(e) => {
                        self.write_output(
                            &self
                                .formatter
                                .format_error(&format!("Shell command failed: {}", e)),
                        )?;
                    }
                }
                Ok(true)
            }
            MetaCommand::Version => {
                self.write_output(&meta_commands::show_version())?;
                Ok(true)
            }
            MetaCommand::Copyright => {
                self.write_output(&meta_commands::show_copyright())?;
                Ok(true)
            }
            MetaCommand::Begin => {
                session_mgr.execute_query("BEGIN TRANSACTION").await?;
                self.write_output("Transaction started.")?;
                Ok(true)
            }
            MetaCommand::Commit => {
                session_mgr.execute_query("COMMIT").await?;
                self.write_output("Transaction committed.")?;
                Ok(true)
            }
            MetaCommand::Rollback => {
                session_mgr.execute_query("ROLLBACK").await?;
                self.write_output("Transaction rolled back.")?;
                Ok(true)
            }
        }
    }

    async fn execute_script(&mut self, path: &str, session_mgr: &mut SessionManager) -> Result<()> {
        let content =
            fs::read_to_string(path).map_err(|_| CliError::ScriptNotFound(path.to_string()))?;

        let commands = self.parse_script(&content);

        for cmd_str in commands {
            let cmd_str = cmd_str.trim();
            if cmd_str.is_empty() || cmd_str.starts_with("--") || cmd_str.starts_with("//") {
                continue;
            }

            let command = crate::command::parser::parse_command(cmd_str);
            match self.execute(command, session_mgr).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    self.write_output(&self.formatter.format_error(&e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    fn parse_script(&self, content: &str) -> Vec<String> {
        let mut commands = Vec::new();
        let mut current = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('\\') {
                if !current.trim().is_empty() {
                    commands.push(current.trim().to_string());
                    current.clear();
                }
                commands.push(trimmed.to_string());
                continue;
            }

            if trimmed.starts_with("--") || trimmed.starts_with("//") {
                continue;
            }

            current.push_str(trimmed);
            current.push(' ');

            if trimmed.ends_with(';') {
                commands.push(current.trim().to_string());
                current.clear();
            }
        }

        if !current.trim().is_empty() {
            commands.push(current.trim().to_string());
        }

        commands
    }

    fn write_output(&mut self, content: &str) -> Result<()> {
        if let Some(ref mut file) = self.output_file {
            file.write_all(content.as_bytes())
                .map_err(CliError::IoError)?;
            file.write_all(b"\n").map_err(CliError::IoError)?;
        } else {
            println!("{}", content);
        }
        Ok(())
    }
}
