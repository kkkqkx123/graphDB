use crate::analysis::explain::ExplainFormat;
use crate::io::{ExportFormat, ImportFormat, ImportTarget};
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
    ForceQuit,
    Help {
        topic: Option<String>,
    },
    Connect {
        space: String,
    },
    Disconnect,
    ConnInfo,
    ShowSpaces,
    ShowTags {
        pattern: Option<String>,
    },
    ShowEdges {
        pattern: Option<String>,
    },
    ShowIndexes {
        pattern: Option<String>,
    },
    ShowUsers,
    ShowFunctions,
    Describe {
        object: String,
    },
    DescribeEdge {
        name: String,
    },
    Format {
        format: OutputFormat,
    },
    Pager {
        command: Option<String>,
    },
    Timing,
    Set {
        name: String,
        value: Option<String>,
    },
    Unset {
        name: String,
    },
    ShowVariables,
    ExecuteScript {
        path: String,
    },
    ExecuteScriptRaw {
        path: String,
    },
    OutputRedirect {
        path: Option<String>,
    },
    ShellCommand {
        command: String,
    },
    Version,
    Copyright,
    Begin,
    Commit,
    Rollback,
    Autocommit {
        value: Option<String>,
    },
    Isolation {
        level: Option<String>,
    },
    Savepoint {
        name: String,
    },
    RollbackTo {
        name: String,
    },
    ReleaseSavepoint {
        name: String,
    },
    TxStatus,
    Edit {
        file: Option<String>,
        line: Option<usize>,
    },
    PrintBuffer,
    ResetBuffer,
    WriteBuffer {
        file: String,
    },
    History {
        action: HistoryAction,
    },
    If {
        condition: String,
    },
    Elif {
        condition: String,
    },
    Else,
    EndIf,
    Explain {
        query: String,
        analyze: bool,
        format: ExplainFormat,
    },
    Profile {
        query: String,
    },
    Import {
        format: ImportFormat,
        file_path: String,
        target: ImportTarget,
        batch_size: Option<usize>,
    },
    Export {
        format: ExportFormat,
        file_path: String,
        query: String,
    },
    Copy {
        direction: CopyDirection,
        target: String,
        file_path: String,
    },
}

#[derive(Debug, Clone)]
pub enum CopyDirection {
    From,
    To,
}

#[derive(Debug, Clone)]
pub enum HistoryAction {
    Show { count: Option<usize> },
    Search { pattern: String },
    Clear,
    Exec { id: usize },
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
        "q!" => Ok(MetaCommand::ForceQuit),
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
        "i" => {
            if arg.is_empty() {
                Err("Usage: \\i <file_path>".to_string())
            } else {
                Ok(MetaCommand::ExecuteScript {
                    path: arg.to_string(),
                })
            }
        }
        "ir" => {
            if arg.is_empty() {
                Err("Usage: \\ir <file_path>".to_string())
            } else {
                Ok(MetaCommand::ExecuteScriptRaw {
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
        "rollback" => {
            let parts: Vec<&str> = arg.split_whitespace().collect();
            if parts.len() >= 2 && parts[0].to_lowercase() == "to" {
                Ok(MetaCommand::RollbackTo {
                    name: parts[1].to_string(),
                })
            } else {
                Ok(MetaCommand::Rollback)
            }
        }
        "autocommit" => {
            let value = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::Autocommit { value })
        }
        "isolation" => {
            let level = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            Ok(MetaCommand::Isolation { level })
        }
        "savepoint" => {
            if arg.is_empty() {
                Err("Usage: \\savepoint <name>".to_string())
            } else {
                let name = arg.split_whitespace().next().unwrap().to_string();
                Ok(MetaCommand::Savepoint { name })
            }
        }
        "release" => {
            if arg.is_empty() {
                Err("Usage: \\release <savepoint_name>".to_string())
            } else {
                let name = arg.split_whitespace().next().unwrap().to_string();
                Ok(MetaCommand::ReleaseSavepoint { name })
            }
        }
        "txstatus" => Ok(MetaCommand::TxStatus),
        "e" | "edit" => {
            let (file, line) = parse_edit_args(arg);
            Ok(MetaCommand::Edit { file, line })
        }
        "p" => Ok(MetaCommand::PrintBuffer),
        "r" => Ok(MetaCommand::ResetBuffer),
        "w" => {
            if arg.is_empty() {
                Err("Usage: \\w <file_path>".to_string())
            } else {
                Ok(MetaCommand::WriteBuffer {
                    file: arg.to_string(),
                })
            }
        }
        "history" => {
            let action = parse_history_action(arg)?;
            Ok(MetaCommand::History { action })
        }
        "if" => {
            if arg.is_empty() {
                Err("Usage: \\if <condition>".to_string())
            } else {
                Ok(MetaCommand::If {
                    condition: arg.to_string(),
                })
            }
        }
        "elif" => {
            if arg.is_empty() {
                Err("Usage: \\elif <condition>".to_string())
            } else {
                Ok(MetaCommand::Elif {
                    condition: arg.to_string(),
                })
            }
        }
        "else" => Ok(MetaCommand::Else),
        "endif" => Ok(MetaCommand::EndIf),
        "explain" => parse_explain_command(arg),
        "profile" => {
            if arg.is_empty() {
                Err("Usage: \\profile <query>".to_string())
            } else {
                Ok(MetaCommand::Profile {
                    query: arg.to_string(),
                })
            }
        }
        "import" => parse_import_command(arg),
        "export" => parse_export_command(arg),
        "copy" => parse_copy_command(arg),
        _ => Err(format!("Unknown command: \\{}", cmd)),
    }
}

fn parse_explain_command(arg: &str) -> Result<MetaCommand, String> {
    if arg.is_empty() {
        return Err("Usage: \\explain [analyze] [format=json|dot] <query>".to_string());
    }

    let mut analyze = false;
    let mut format = ExplainFormat::Text;
    let mut query_parts = Vec::new();

    for part in arg.split_whitespace() {
        if part.to_lowercase() == "analyze" {
            analyze = true;
        } else if part.to_lowercase().starts_with("format=") {
            let fmt = part.split('=').nth(1).unwrap_or("text");
            format = ExplainFormat::from_str(fmt);
        } else {
            query_parts.push(part);
        }
    }

    if query_parts.is_empty() {
        return Err("Usage: \\explain [analyze] [format=json|dot] <query>".to_string());
    }

    Ok(MetaCommand::Explain {
        query: query_parts.join(" "),
        analyze,
        format,
    })
}

fn parse_import_command(arg: &str) -> Result<MetaCommand, String> {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    
    if parts.len() < 4 {
        return Err("Usage: \\import <csv|json|jsonl> <file> <tag|edge> <name> [batch_size]".to_string());
    }

    let format = match parts[0].to_lowercase().as_str() {
        "csv" => ImportFormat::csv(),
        "json" => ImportFormat::json_array(),
        "jsonl" => ImportFormat::json_lines(),
        _ => return Err(format!("Unsupported format: {}", parts[0])),
    };

    let file_path = parts[1].to_string();

    let target = match parts[2].to_lowercase().as_str() {
        "tag" | "vertex" => ImportTarget::vertex(parts[3]),
        "edge" => ImportTarget::edge(parts[3]),
        _ => return Err(format!("Invalid target type: {}", parts[2])),
    };

    let batch_size = parts.get(4).and_then(|s| s.parse().ok());

    Ok(MetaCommand::Import {
        format,
        file_path,
        target,
        batch_size,
    })
}

fn parse_export_command(arg: &str) -> Result<MetaCommand, String> {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    
    if parts.len() < 3 {
        return Err("Usage: \\export <csv|json|jsonl> <file> <query>".to_string());
    }

    let format = match parts[0].to_lowercase().as_str() {
        "csv" => ExportFormat::csv(),
        "json" => ExportFormat::json(),
        "jsonl" => ExportFormat::json_lines(),
        _ => return Err(format!("Unsupported format: {}", parts[0])),
    };

    let file_path = parts[1].to_string();
    let query = parts[2..].join(" ");

    Ok(MetaCommand::Export {
        format,
        file_path,
        query,
    })
}

fn parse_copy_command(arg: &str) -> Result<MetaCommand, String> {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    
    if parts.len() < 4 {
        return Err("Usage: \\copy <target> from|to '<file>'".to_string());
    }

    let target = parts[0].to_string();
    let direction = match parts[1].to_lowercase().as_str() {
        "from" => CopyDirection::From,
        "to" => CopyDirection::To,
        _ => return Err(format!("Invalid direction: {}. Use 'from' or 'to'", parts[1])),
    };
    
    let file_path = parts[2].trim_matches('\'').to_string();

    Ok(MetaCommand::Copy {
        direction,
        target,
        file_path,
    })
}

fn parse_edit_args(arg: &str) -> (Option<String>, Option<usize>) {
    if arg.is_empty() {
        return (None, None);
    }

    let parts: Vec<&str> = arg.split_whitespace().collect();
    let mut file = None;
    let mut line = None;

    for part in parts {
        if let Some(l) = part.strip_prefix('+') {
            line = l.parse().ok();
        } else {
            file = Some(part.to_string());
        }
    }

    (file, line)
}

fn parse_history_action(arg: &str) -> Result<HistoryAction, String> {
    if arg.is_empty() {
        return Ok(HistoryAction::Show { count: Some(20) });
    }

    let parts: Vec<&str> = arg.splitn(2, char::is_whitespace).collect();
    let subcmd = parts[0].to_lowercase();
    let sub_arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match subcmd.as_str() {
        "clear" => Ok(HistoryAction::Clear),
        "search" => {
            if sub_arg.is_empty() {
                Err("Usage: \\history search <pattern>".to_string())
            } else {
                Ok(HistoryAction::Search {
                    pattern: sub_arg.to_string(),
                })
            }
        }
        "exec" => {
            if sub_arg.is_empty() {
                Err("Usage: \\history exec <id>".to_string())
            } else {
                let id = sub_arg
                    .parse()
                    .map_err(|_| format!("Invalid history ID: {}", sub_arg))?;
                Ok(HistoryAction::Exec { id })
            }
        }
        n => {
            if let Ok(count) = n.parse::<usize>() {
                Ok(HistoryAction::Show { count: Some(count) })
            } else {
                Err(format!("Unknown history action: {}", n))
            }
        }
    }
}

fn whitespace_or_end(c: char) -> bool {
    c.is_whitespace()
}
