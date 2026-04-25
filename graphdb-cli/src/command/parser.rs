use crate::output::formatter::OutputFormat;

#[derive(Debug)]
pub enum Command {
    Query(String),
    MetaCommand(MetaCommand),
    Empty,
}

#[derive(Debug)]
pub enum MetaCommand {
    Quit,
    Help { topic: Option<String> },
    Connect { space: String },
    Disconnect,
    ConnInfo,
    ShowSpaces,
    ShowTags { pattern: Option<String> },
    ShowEdges { pattern: Option<String> },
    ShowIndexes { pattern: Option<String> },
    ShowUsers,
    ShowFunctions,
    Describe { object: String },
    DescribeEdge { name: String },
    Format { format: OutputFormat },
    Pager { command: Option<String> },
    Timing,
    Set { name: String, value: Option<String> },
    Unset { name: String },
    ShowVariables,
    ExecuteScript { path: String },
    OutputRedirect { path: Option<String> },
    ShellCommand { command: String },
    Version,
    Copyright,
    Begin,
    Commit,
    Rollback,
}

pub fn parse_command(input: &str) -> Command {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Command::Empty;
    }

    if trimmed.starts_with('\\') {
        match parse_meta_command(trimmed) {
            Ok(cmd) => Command::MetaCommand(cmd),
            Err(msg) => Command::MetaCommand(MetaCommand::Help { topic: Some(msg) }),
        }
    } else {
        Command::Query(trimmed.to_string())
    }
}

fn parse_meta_command(input: &str) -> Result<MetaCommand, String> {
    let trimmed = input.trim_start_matches('\\');
    let parts: Vec<&str> = trimmed.splitn(2, whitespace_or_end).collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd.as_str() {
        "q" | "quit" => Ok(MetaCommand::Quit),
        "?" => Ok(MetaCommand::Help { topic: None }),
        "help" => {
            let topic = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::Help { topic })
        }
        "connect" | "c" => {
            if arg.is_empty() {
                Err("Usage: \\connect <space_name>".to_string())
            } else {
                Ok(MetaCommand::Connect {
                    space: arg.to_string(),
                })
            }
        }
        "disconnect" => Ok(MetaCommand::Disconnect),
        "conninfo" => Ok(MetaCommand::ConnInfo),
        "show_spaces" | "l" => Ok(MetaCommand::ShowSpaces),
        "show_tags" | "dt" => {
            let pattern = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::ShowTags { pattern })
        }
        "show_edges" | "de" => {
            let pattern = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::ShowEdges { pattern })
        }
        "show_indexes" | "di" => {
            let pattern = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::ShowIndexes { pattern })
        }
        "show_users" | "du" => Ok(MetaCommand::ShowUsers),
        "show_functions" | "df" => Ok(MetaCommand::ShowFunctions),
        "describe" | "d" => {
            if arg.is_empty() {
                Err("Usage: \\describe <tag_name>".to_string())
            } else {
                Ok(MetaCommand::Describe {
                    object: arg.to_string(),
                })
            }
        }
        "describe_edge" => {
            if arg.is_empty() {
                Err("Usage: \\describe_edge <edge_name>".to_string())
            } else {
                Ok(MetaCommand::DescribeEdge {
                    name: arg.to_string(),
                })
            }
        }
        "format" => {
            if arg.is_empty() {
                Err("Usage: \\format <table|csv|json|vertical|html>".to_string())
            } else {
                match OutputFormat::parse(arg) {
                    Some(fmt) => Ok(MetaCommand::Format { format: fmt }),
                    None => Err(format!(
                        "Unknown format: '{}'. Available: table, csv, json, vertical, html",
                        arg
                    )),
                }
            }
        }
        "pager" => {
            let command = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::Pager { command })
        }
        "timing" => Ok(MetaCommand::Timing),
        "set" => {
            let set_parts: Vec<&str> = arg.splitn(2, char::is_whitespace).collect();
            if set_parts.is_empty() || set_parts[0].is_empty() {
                Ok(MetaCommand::ShowVariables)
            } else {
                let name = set_parts[0].to_string();
                let value = set_parts.get(1).map(|s| s.to_string());
                Ok(MetaCommand::Set { name, value })
            }
        }
        "unset" => {
            if arg.is_empty() {
                Err("Usage: \\unset <variable_name>".to_string())
            } else {
                Ok(MetaCommand::Unset {
                    name: arg.to_string(),
                })
            }
        }
        "i" | "ir" => {
            if arg.is_empty() {
                Err("Usage: \\i <file_path>".to_string())
            } else {
                Ok(MetaCommand::ExecuteScript {
                    path: arg.to_string(),
                })
            }
        }
        "o" => {
            let path = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::OutputRedirect { path })
        }
        "!" => {
            if arg.is_empty() {
                Err("Usage: \\! <shell_command>".to_string())
            } else {
                Ok(MetaCommand::ShellCommand {
                    command: arg.to_string(),
                })
            }
        }
        "version" => Ok(MetaCommand::Version),
        "copyright" => Ok(MetaCommand::Copyright),
        "x" => Ok(MetaCommand::Format {
            format: OutputFormat::Vertical,
        }),
        "begin" => Ok(MetaCommand::Begin),
        "commit" => Ok(MetaCommand::Commit),
        "rollback" => Ok(MetaCommand::Rollback),
        _ => Err(format!("Unknown command: \\{}", cmd)),
    }
}

fn whitespace_or_end(c: char) -> bool {
    c.is_whitespace()
}
