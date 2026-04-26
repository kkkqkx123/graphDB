use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;

use crate::analysis::timing::QueryTimer;
use crate::command::meta_commands;
use crate::command::parser::{Command, CopyDirection, HistoryAction, MetaCommand};
use crate::command::script::{
    ConditionExpr, ConditionalStack, ScriptExecutionContext, ScriptParser,
};
use crate::input::buffer::{self, QueryBuffer};
use crate::io::{CsvExporter, CsvImporter, ExportConfig, ImportConfig, JsonExporter, JsonImporter};
use crate::output::formatter::OutputFormatter;
use crate::output::pager::Pager;
use crate::session::manager::SessionManager;
use crate::transaction::{IsolationLevel, TransactionManager};
use crate::utils::error::{CliError, Result};

pub struct CommandExecutor {
    formatter: OutputFormatter,
    pager: Pager,
    output_file: Option<std::fs::File>,
    query_buffer: QueryBuffer,
    conditional_stack: ConditionalStack,
    script_ctx: ScriptExecutionContext,
    force_mode: bool,
    single_transaction: bool,
    transaction_active: bool,
    tx_manager: TransactionManager,
}

impl CommandExecutor {
    pub fn new(formatter: OutputFormatter) -> Self {
        Self {
            formatter,
            pager: Pager::new(),
            output_file: None,
            query_buffer: QueryBuffer::new(),
            conditional_stack: ConditionalStack::new(),
            script_ctx: ScriptExecutionContext::new(),
            force_mode: false,
            single_transaction: false,
            transaction_active: false,
            tx_manager: TransactionManager::new(),
        }
    }

    pub fn with_options(formatter: OutputFormatter, force: bool, single_transaction: bool) -> Self {
        Self {
            formatter,
            pager: Pager::new(),
            output_file: None,
            query_buffer: QueryBuffer::new(),
            conditional_stack: ConditionalStack::new(),
            script_ctx: ScriptExecutionContext::new(),
            force_mode: force,
            single_transaction,
            transaction_active: false,
            tx_manager: TransactionManager::new(),
        }
    }

    pub fn formatter(&self) -> &OutputFormatter {
        &self.formatter
    }

    pub fn formatter_mut(&mut self) -> &mut OutputFormatter {
        &mut self.formatter
    }

    pub fn query_buffer(&self) -> &QueryBuffer {
        &self.query_buffer
    }

    pub fn query_buffer_mut(&mut self) -> &mut QueryBuffer {
        &mut self.query_buffer
    }

    pub fn conditional_stack(&self) -> &ConditionalStack {
        &self.conditional_stack
    }

    pub fn tx_manager(&self) -> &TransactionManager {
        &self.tx_manager
    }

    pub fn tx_manager_mut(&mut self) -> &mut TransactionManager {
        &mut self.tx_manager
    }

    pub fn set_force_mode(&mut self, force: bool) {
        self.force_mode = force;
    }

    pub fn set_single_transaction(&mut self, single: bool) {
        self.single_transaction = single;
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

    pub fn execute_meta_sync(
        &mut self,
        meta: MetaCommand,
        session_mgr: &mut SessionManager,
    ) -> Result<SyncMetaResult> {
        match &meta {
            MetaCommand::If { .. }
            | MetaCommand::Elif { .. }
            | MetaCommand::Else
            | MetaCommand::EndIf => {
                self.handle_conditional(&meta, session_mgr)?;
                Ok(SyncMetaResult::Continue)
            }
            MetaCommand::Edit { .. }
            | MetaCommand::PrintBuffer
            | MetaCommand::ResetBuffer
            | MetaCommand::WriteBuffer { .. } => {
                self.handle_buffer_command(&meta, session_mgr)?;
                Ok(SyncMetaResult::Continue)
            }
            MetaCommand::History { .. } => {
                self.handle_history_command(&meta, session_mgr)?;
                Ok(SyncMetaResult::Continue)
            }
            _ => Ok(SyncMetaResult::NeedsAsync(meta)),
        }
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

        if !self.conditional_stack.is_active() {
            return Ok(true);
        }

        if self.tx_manager.is_failed() {
            let error = self
                .tx_manager
                .state()
                .error_message()
                .unwrap_or("Transaction is in failed state")
                .to_string();
            return Err(CliError::TransactionFailed(error));
        }

        let use_match = query.trim_start().to_uppercase();
        if use_match.starts_with("USE ") {
            let space = query.trim_start()[4..].trim().trim_end_matches(';').trim();
            return self.execute_use_space(space, session_mgr).await;
        }

        let mut timer = QueryTimer::new();

        let result = session_mgr.execute_query(query).await?;
        timer.record_phase("execution");

        self.tx_manager.record_query();

        let output = self.formatter.format_result(&result);
        self.write_output(&output)?;

        if self.formatter.timing_enabled() {
            self.write_output(&timer.format_time())?;
        }

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
                if self.tx_manager.is_active() {
                    self.write_output("WARNING: There is an active transaction. Use \\commit or \\rollback before quitting, or \\q! to force quit.")?;
                    return Ok(true);
                }
                self.write_output("Goodbye!")?;
                Ok(false)
            }
            MetaCommand::ForceQuit => {
                if self.tx_manager.is_active() {
                    self.write_output("Rolling back active transaction...")?;
                    let _ = self.tx_manager.rollback(session_mgr).await;
                }
                self.write_output("Goodbye!")?;
                Ok(false)
            }
            MetaCommand::Help { topic } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let help = meta_commands::show_help(topic.as_deref());
                self.write_output(&help)?;
                Ok(true)
            }
            MetaCommand::Connect { space } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                session_mgr.switch_space(&space).await?;
                self.write_output(&format!("Connected to space '{}'", space))?;
                Ok(true)
            }
            MetaCommand::Disconnect => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                session_mgr.disconnect().await?;
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
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let spaces = session_mgr.list_spaces().await?;
                let output = self.formatter.format_spaces(&spaces);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::ShowTags { .. } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let tags = session_mgr.list_tags().await?;
                let output = self.formatter.format_tags(&tags);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::ShowEdges { .. } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let edge_types = session_mgr.list_edge_types().await?;
                let output = self.formatter.format_edge_types(&edge_types);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::ShowIndexes { .. } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.write_output("Index listing is not yet supported via CLI.")?;
                Ok(true)
            }
            MetaCommand::ShowUsers => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.write_output("User listing is not yet supported via CLI.")?;
                Ok(true)
            }
            MetaCommand::ShowFunctions => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.write_output("Function listing is not yet supported via CLI.")?;
                Ok(true)
            }
            MetaCommand::Describe { object } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
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
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
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
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let session = session_mgr.session_mut().ok_or(CliError::NotConnected)?;
                match value {
                    Some(v) => {
                        session.set_variable(name.clone(), v)?;
                        let val = session.get_variable(&name).unwrap();
                        self.write_output(&format!("Set variable: {} = {}", name, val))?;
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
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let session = session_mgr.session_mut().ok_or(CliError::NotConnected)?;
                session.remove_variable(&name);
                self.write_output(&format!("Unset variable: {}", name))?;
                Ok(true)
            }
            MetaCommand::ShowVariables => {
                let session = session_mgr.session().ok_or(CliError::NotConnected)?;
                let all_vars = session.variable_store.all_variables();
                if all_vars.is_empty() {
                    self.write_output("(no variables set)")?;
                } else {
                    let mut output = String::new();
                    for (name, value) in all_vars {
                        let marker = if session.variable_store.is_special(name) {
                            "*"
                        } else {
                            ""
                        };
                        output.push_str(&format!("{}{} = {}\n", marker, name, value));
                    }
                    self.write_output(output.trim_end())?;
                }
                Ok(true)
            }
            MetaCommand::ExecuteScript { path } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.execute_script(&path, session_mgr, false).await?;
                Ok(true)
            }
            MetaCommand::ExecuteScriptRaw { path } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.execute_script(&path, session_mgr, true).await?;
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
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
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
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.tx_manager.begin(session_mgr).await?;
                self.write_output("Transaction started.")?;
                Ok(true)
            }
            MetaCommand::Commit => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.tx_manager.commit(session_mgr).await?;
                self.write_output("Transaction committed.")?;
                Ok(true)
            }
            MetaCommand::Rollback => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.tx_manager.rollback(session_mgr).await?;
                self.write_output("Transaction rolled back.")?;
                Ok(true)
            }
            MetaCommand::Autocommit { value } => {
                if let Some(v) = value {
                    let enabled = match v.to_lowercase().as_str() {
                        "on" | "true" | "1" => true,
                        "off" | "false" | "0" => false,
                        _ => {
                            return Err(CliError::InvalidValue(format!(
                                "Invalid autocommit value: {}",
                                v
                            )))
                        }
                    };

                    if !enabled && self.tx_manager.is_active() {
                        return Err(CliError::CannotChangeAutocommit);
                    }

                    self.tx_manager.set_autocommit(enabled);
                    self.write_output(&format!(
                        "Autocommit {}.",
                        if enabled { "enabled" } else { "disabled" }
                    ))?;
                } else {
                    self.write_output(&format!(
                        "Autocommit is {}.",
                        if self.tx_manager.autocommit() {
                            "on"
                        } else {
                            "off"
                        }
                    ))?;
                }
                Ok(true)
            }
            MetaCommand::Isolation { level } => {
                if let Some(l) = level {
                    let isolation = IsolationLevel::from_str(&l).ok_or_else(|| {
                        CliError::InvalidValue(format!("Invalid isolation level: {}", l))
                    })?;

                    if self.tx_manager.is_active() {
                        return Err(CliError::TransactionAlreadyActive);
                    }

                    self.tx_manager.set_isolation_level(isolation);
                    self.write_output(&format!("Isolation level set to: {}", isolation.as_str()))?;
                } else {
                    self.write_output(&format!(
                        "Current isolation level: {}",
                        self.tx_manager.isolation_level().as_str()
                    ))?;
                }
                Ok(true)
            }
            MetaCommand::Savepoint { name } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.tx_manager.create_savepoint(&name, session_mgr).await?;
                self.write_output(&format!("Savepoint '{}' created.", name))?;
                Ok(true)
            }
            MetaCommand::RollbackTo { name } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.tx_manager
                    .rollback_to_savepoint(&name, session_mgr)
                    .await?;
                self.write_output(&format!("Rolled back to savepoint '{}'.", name))?;
                Ok(true)
            }
            MetaCommand::ReleaseSavepoint { name } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                self.tx_manager
                    .release_savepoint(&name, session_mgr)
                    .await?;
                self.write_output(&format!("Savepoint '{}' released.", name))?;
                Ok(true)
            }
            MetaCommand::TxStatus => {
                let info = self.tx_manager.info();
                self.write_output(&info.format_status())?;
                Ok(true)
            }
            MetaCommand::Edit { file, line } => {
                let editor = session_mgr
                    .session()
                    .and_then(|s| s.get_variable("EDITOR").cloned());
                let editor_ref = editor.as_deref();
                let result = buffer::edit_in_external_editor(
                    &mut self.query_buffer,
                    file.as_deref(),
                    line,
                    editor_ref,
                )?;
                if result {
                    let content = self.query_buffer.content();
                    if !content.trim().is_empty() {
                        self.write_output(&content)?;
                    }
                }
                Ok(true)
            }
            MetaCommand::PrintBuffer => {
                let content = self.query_buffer.content();
                if content.trim().is_empty() {
                    self.write_output("(query buffer is empty)")?;
                } else {
                    self.write_output(&content)?;
                }
                Ok(true)
            }
            MetaCommand::ResetBuffer => {
                self.query_buffer.reset();
                self.write_output("Query buffer reset.")?;
                Ok(true)
            }
            MetaCommand::WriteBuffer { file } => {
                buffer::write_buffer_to_file(&self.query_buffer, &file)?;
                self.write_output(&format!("Query buffer written to: {}", file))?;
                Ok(true)
            }
            MetaCommand::History { action } => {
                self.handle_history_action(action, session_mgr)?;
                Ok(true)
            }
            MetaCommand::If { condition } => {
                self.handle_if(condition, session_mgr)?;
                Ok(true)
            }
            MetaCommand::Elif { condition } => {
                self.handle_elif(condition, session_mgr)?;
                Ok(true)
            }
            MetaCommand::Else => {
                self.conditional_stack.push_else();
                Ok(true)
            }
            MetaCommand::EndIf => {
                self.conditional_stack.pop();
                Ok(true)
            }
            MetaCommand::Explain {
                query,
                analyze,
                format: _,
            } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                if analyze {
                    self.write_output(
                        "EXPLAIN ANALYZE is not yet implemented. Showing plan only.",
                    )?;
                }
                let result = session_mgr
                    .execute_query(&format!("EXPLAIN {}", query))
                    .await?;
                let output = self.formatter.format_result(&result);
                self.write_output(&output)?;
                Ok(true)
            }
            MetaCommand::Profile { query } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }
                let mut timer = QueryTimer::new();
                let result = session_mgr.execute_query(&query).await?;
                timer.record_phase("execution");

                let output = self.formatter.format_result(&result);
                self.write_output(&output)?;
                self.write_output(&timer.format_phases())?;
                Ok(true)
            }
            MetaCommand::Import {
                format,
                file_path,
                target,
                batch_size,
            } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }

                let config = ImportConfig::new(file_path.into(), target)
                    .with_format(format)
                    .with_batch_size(batch_size.unwrap_or(100));

                let stats = match config.format {
                    crate::io::ImportFormat::Csv { .. } => {
                        let mut importer = CsvImporter::new(config);
                        importer.import(session_mgr).await?
                    }
                    crate::io::ImportFormat::Json { .. } => {
                        let mut importer = JsonImporter::new(config);
                        importer.import(session_mgr).await?
                    }
                    crate::io::ImportFormat::JsonLines => {
                        let mut importer = JsonImporter::new(config);
                        importer.import(session_mgr).await?
                    }
                };

                self.write_output(&stats.format_summary())?;
                Ok(true)
            }
            MetaCommand::Export {
                format,
                file_path,
                query,
            } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }

                let config = ExportConfig::new(file_path.into(), format);

                let stats = match &config.format {
                    crate::io::ExportFormat::Csv { .. } => {
                        let exporter = CsvExporter::new(config);
                        exporter.export(&query, session_mgr).await?
                    }
                    crate::io::ExportFormat::Json { .. } | crate::io::ExportFormat::JsonLines => {
                        let exporter = JsonExporter::new(config);
                        exporter.export(&query, session_mgr).await?
                    }
                };

                self.write_output(&stats.format_summary())?;
                Ok(true)
            }
            MetaCommand::Copy {
                direction,
                target,
                file_path,
            } => {
                if !self.conditional_stack.is_active() {
                    return Ok(true);
                }

                match direction {
                    CopyDirection::From => {
                        let import_format =
                            if file_path.ends_with(".json") || file_path.ends_with(".jsonl") {
                                crate::io::ImportFormat::json_array()
                            } else {
                                crate::io::ImportFormat::csv()
                            };

                        let config = ImportConfig::new(
                            file_path.into(),
                            crate::io::ImportTarget::vertex(&target),
                        )
                        .with_format(import_format.clone());

                        let stats = match import_format {
                            crate::io::ImportFormat::Csv { .. } => {
                                let mut importer = CsvImporter::new(config);
                                importer.import(session_mgr).await?
                            }
                            _ => {
                                let mut importer = JsonImporter::new(config);
                                importer.import(session_mgr).await?
                            }
                        };

                        self.write_output(&stats.format_summary())?;
                    }
                    CopyDirection::To => {
                        let query = format!("MATCH (n:{}) RETURN n", target);
                        let export_format = if file_path.ends_with(".json") {
                            crate::io::ExportFormat::json()
                        } else {
                            crate::io::ExportFormat::csv()
                        };

                        let config = ExportConfig::new(file_path.into(), export_format);

                        let stats = match &config.format {
                            crate::io::ExportFormat::Csv { .. } => {
                                let exporter = CsvExporter::new(config);
                                exporter.export(&query, session_mgr).await?
                            }
                            _ => {
                                let exporter = JsonExporter::new(config);
                                exporter.export(&query, session_mgr).await?
                            }
                        };

                        self.write_output(&stats.format_summary())?;
                    }
                }
                Ok(true)
            }
        }
    }

    fn handle_conditional(
        &mut self,
        meta: &MetaCommand,
        session_mgr: &mut SessionManager,
    ) -> Result<()> {
        match meta {
            MetaCommand::If { condition } => self.handle_if(condition.clone(), session_mgr)?,
            MetaCommand::Elif { condition } => self.handle_elif(condition.clone(), session_mgr)?,
            MetaCommand::Else => self.conditional_stack.push_else(),
            MetaCommand::EndIf => {
                self.conditional_stack.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_if(&mut self, condition: String, session_mgr: &mut SessionManager) -> Result<()> {
        let vars = self.get_all_variables(session_mgr);
        let expr = ConditionExpr::parse(&condition)?;
        let result = expr.evaluate(&vars);
        self.conditional_stack.push_if(result);
        Ok(())
    }

    fn handle_elif(&mut self, condition: String, session_mgr: &mut SessionManager) -> Result<()> {
        let vars = self.get_all_variables(session_mgr);
        let expr = ConditionExpr::parse(&condition)?;
        let result = expr.evaluate(&vars);
        self.conditional_stack.push_elif(result);
        Ok(())
    }

    fn get_all_variables(&self, session_mgr: &SessionManager) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        for (key, val) in std::env::vars() {
            vars.insert(format!("ENV_{}", key), val);
        }

        if let Some(session) = session_mgr.session() {
            for (key, val) in session.variable_store.all_variables() {
                vars.insert(key.clone(), val.clone());
            }
        }

        vars
    }

    fn handle_buffer_command(
        &mut self,
        meta: &MetaCommand,
        session_mgr: &mut SessionManager,
    ) -> Result<()> {
        match meta {
            MetaCommand::Edit { file, line } => {
                let editor = session_mgr
                    .session()
                    .and_then(|s| s.get_variable("EDITOR").cloned());
                let editor_ref = editor.as_deref();
                let result = buffer::edit_in_external_editor(
                    &mut self.query_buffer,
                    file.as_deref(),
                    *line,
                    editor_ref,
                )?;
                if result {
                    let content = self.query_buffer.content();
                    if !content.trim().is_empty() {
                        self.write_output(&content)?;
                    }
                }
            }
            MetaCommand::PrintBuffer => {
                let content = self.query_buffer.content();
                if content.trim().is_empty() {
                    self.write_output("(query buffer is empty)")?;
                } else {
                    self.write_output(&content)?;
                }
            }
            MetaCommand::ResetBuffer => {
                self.query_buffer.reset();
                self.write_output("Query buffer reset.")?;
            }
            MetaCommand::WriteBuffer { file } => {
                buffer::write_buffer_to_file(&self.query_buffer, file)?;
                self.write_output(&format!("Query buffer written to: {}", file))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_history_command(
        &mut self,
        meta: &MetaCommand,
        _session_mgr: &mut SessionManager,
    ) -> Result<()> {
        if let MetaCommand::History { action } = meta {
            self.handle_history_action(action.clone(), _session_mgr)?;
        }
        Ok(())
    }

    fn handle_history_action(
        &mut self,
        action: HistoryAction,
        _session_mgr: &mut SessionManager,
    ) -> Result<()> {
        match action {
            HistoryAction::Show { count } => {
                self.write_output("History display is handled by the REPL loop.")?;
                let _ = count;
            }
            HistoryAction::Search { pattern } => {
                self.write_output(&format!(
                    "History search for '{}' is handled by the REPL loop.",
                    pattern
                ))?;
            }
            HistoryAction::Clear => {
                self.write_output("History clear is handled by the REPL loop.")?;
            }
            HistoryAction::Exec { id } => {
                self.write_output(&format!(
                    "History exec #{} is handled by the REPL loop.",
                    id
                ))?;
                let _ = id;
            }
        }
        Ok(())
    }

    async fn execute_script(
        &mut self,
        path: &str,
        session_mgr: &mut SessionManager,
        raw: bool,
    ) -> Result<()> {
        self.script_ctx.enter_script(path)?;

        let content =
            fs::read_to_string(path).map_err(|_| CliError::ScriptNotFound(path.to_string()))?;

        let statements = ScriptParser::parse(&content);

        if self.single_transaction && !self.transaction_active {
            session_mgr.execute_query("BEGIN TRANSACTION").await?;
            self.transaction_active = true;
        }

        for stmt in &statements {
            if !self.conditional_stack.is_active()
                && !matches!(
                    stmt.kind,
                    crate::command::script::StatementKind::MetaCommand
                )
            {
                continue;
            }

            let content = if !raw {
                let session = session_mgr.session();
                if let Some(s) = session {
                    s.substitute_variables(&stmt.content)?
                } else {
                    stmt.content.clone()
                }
            } else {
                stmt.content.clone()
            };

            let command = crate::command::parser::parse_command(&content);
            match self.execute(command, session_mgr).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    let line_info = if stmt.start_line == stmt.end_line {
                        format!("line {}", stmt.start_line)
                    } else {
                        format!("lines {}-{}", stmt.start_line, stmt.end_line)
                    };
                    self.write_output(
                        &self
                            .formatter
                            .format_error(&format!("{}: {} (in {})", path, e, line_info)),
                    )?;

                    let on_error_stop = session_mgr
                        .session()
                        .map(|s| s.variable_store.get_bool("ON_ERROR_STOP"))
                        .unwrap_or(false);

                    if on_error_stop && !self.force_mode {
                        break;
                    }
                }
            }
        }

        if self.single_transaction && self.transaction_active {
            match session_mgr.execute_query("COMMIT").await {
                Ok(_) => {
                    self.transaction_active = false;
                    self.write_output("Transaction committed.")?;
                }
                Err(e) => {
                    let _ = session_mgr.execute_query("ROLLBACK").await;
                    self.transaction_active = false;
                    self.write_output(
                        &self
                            .formatter
                            .format_error(&format!("Transaction failed, rolled back: {}", e)),
                    )?;
                }
            }
        }

        self.script_ctx.exit_script();
        Ok(())
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

pub enum SyncMetaResult {
    Continue,
    NeedsAsync(MetaCommand),
}
