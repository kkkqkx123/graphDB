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
        return Err(
            "Usage: \\import <csv|json|jsonl> <file> <tag|edge> <name> [batch_size]".to_string(),
        );
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
        _ => {
            return Err(format!(
                "Invalid direction: {}. Use 'from' or 'to'",
                parts[1]
            ))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_empty() {
        assert!(matches!(parse_command(""), Command::Empty));
        assert!(matches!(parse_command("   "), Command::Empty));
        assert!(matches!(parse_command("\t\n"), Command::Empty));
    }

    #[test]
    fn test_parse_command_query() {
        let query = "MATCH (v:Person) RETURN v";
        match parse_command(query) {
            Command::Query(q) => assert_eq!(q, query),
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_parse_command_query_trimmed() {
        match parse_command("  MATCH (v)  ") {
            Command::Query(q) => assert_eq!(q, "MATCH (v)"),
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_parse_meta_command_quit() {
        assert!(matches!(
            parse_command("\\q"),
            Command::MetaCommand(MetaCommand::Quit)
        ));
        assert!(matches!(
            parse_command("\\quit"),
            Command::MetaCommand(MetaCommand::Quit)
        ));
    }

    #[test]
    fn test_parse_meta_command_force_quit() {
        assert!(matches!(
            parse_command("\\q!"),
            Command::MetaCommand(MetaCommand::ForceQuit)
        ));
    }

    #[test]
    fn test_parse_meta_command_help() {
        match parse_command("\\?") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_none());
            }
            _ => panic!("Expected Help command"),
        }

        match parse_command("\\help match") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert_eq!(topic, Some("match".to_string()));
            }
            _ => panic!("Expected Help command with topic"),
        }
    }

    #[test]
    fn test_parse_meta_command_connect() {
        match parse_command("\\connect myspace") {
            Command::MetaCommand(MetaCommand::Connect { space }) => {
                assert_eq!(space, "myspace");
            }
            _ => panic!("Expected Connect command"),
        }

        match parse_command("\\c myspace") {
            Command::MetaCommand(MetaCommand::Connect { space }) => {
                assert_eq!(space, "myspace");
            }
            _ => panic!("Expected Connect command with alias"),
        }
    }

    #[test]
    fn test_parse_meta_command_connect_error() {
        match parse_command("\\connect") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_some());
                assert!(topic.unwrap().contains("Usage"));
            }
            _ => panic!("Expected Help command with error message"),
        }
    }

    #[test]
    fn test_parse_meta_command_disconnect() {
        assert!(matches!(
            parse_command("\\disconnect"),
            Command::MetaCommand(MetaCommand::Disconnect)
        ));
    }

    #[test]
    fn test_parse_meta_command_conninfo() {
        assert!(matches!(
            parse_command("\\conninfo"),
            Command::MetaCommand(MetaCommand::ConnInfo)
        ));
    }

    #[test]
    fn test_parse_meta_command_show_spaces() {
        assert!(matches!(
            parse_command("\\show_spaces"),
            Command::MetaCommand(MetaCommand::ShowSpaces)
        ));
        assert!(matches!(
            parse_command("\\l"),
            Command::MetaCommand(MetaCommand::ShowSpaces)
        ));
    }

    #[test]
    fn test_parse_meta_command_show_tags() {
        match parse_command("\\show_tags") {
            Command::MetaCommand(MetaCommand::ShowTags { pattern }) => {
                assert!(pattern.is_none());
            }
            _ => panic!("Expected ShowTags command"),
        }

        match parse_command("\\dt person*") {
            Command::MetaCommand(MetaCommand::ShowTags { pattern }) => {
                assert_eq!(pattern, Some("person*".to_string()));
            }
            _ => panic!("Expected ShowTags command with pattern"),
        }
    }

    #[test]
    fn test_parse_meta_command_show_edges() {
        match parse_command("\\show_edges") {
            Command::MetaCommand(MetaCommand::ShowEdges { pattern }) => {
                assert!(pattern.is_none());
            }
            _ => panic!("Expected ShowEdges command"),
        }

        match parse_command("\\de friend*") {
            Command::MetaCommand(MetaCommand::ShowEdges { pattern }) => {
                assert_eq!(pattern, Some("friend*".to_string()));
            }
            _ => panic!("Expected ShowEdges command with pattern"),
        }
    }

    #[test]
    fn test_parse_meta_command_show_indexes() {
        match parse_command("\\show_indexes") {
            Command::MetaCommand(MetaCommand::ShowIndexes { pattern }) => {
                assert!(pattern.is_none());
            }
            _ => panic!("Expected ShowIndexes command"),
        }

        match parse_command("\\di idx*") {
            Command::MetaCommand(MetaCommand::ShowIndexes { pattern }) => {
                assert_eq!(pattern, Some("idx*".to_string()));
            }
            _ => panic!("Expected ShowIndexes command with pattern"),
        }
    }

    #[test]
    fn test_parse_meta_command_show_users() {
        assert!(matches!(
            parse_command("\\show_users"),
            Command::MetaCommand(MetaCommand::ShowUsers)
        ));
        assert!(matches!(
            parse_command("\\du"),
            Command::MetaCommand(MetaCommand::ShowUsers)
        ));
    }

    #[test]
    fn test_parse_meta_command_show_functions() {
        assert!(matches!(
            parse_command("\\show_functions"),
            Command::MetaCommand(MetaCommand::ShowFunctions)
        ));
        assert!(matches!(
            parse_command("\\df"),
            Command::MetaCommand(MetaCommand::ShowFunctions)
        ));
    }

    #[test]
    fn test_parse_meta_command_describe() {
        match parse_command("\\describe Person") {
            Command::MetaCommand(MetaCommand::Describe { object }) => {
                assert_eq!(object, "Person");
            }
            _ => panic!("Expected Describe command"),
        }

        match parse_command("\\d Person") {
            Command::MetaCommand(MetaCommand::Describe { object }) => {
                assert_eq!(object, "Person");
            }
            _ => panic!("Expected Describe command with alias"),
        }
    }

    #[test]
    fn test_parse_meta_command_describe_edge() {
        match parse_command("\\describe_edge Friend") {
            Command::MetaCommand(MetaCommand::DescribeEdge { name }) => {
                assert_eq!(name, "Friend");
            }
            _ => panic!("Expected DescribeEdge command"),
        }
    }

    #[test]
    fn test_parse_meta_command_format() {
        use crate::output::formatter::OutputFormat;

        match parse_command("\\format table") {
            Command::MetaCommand(MetaCommand::Format { format }) => {
                assert!(matches!(format, OutputFormat::Table));
            }
            _ => panic!("Expected Format command"),
        }

        match parse_command("\\format json") {
            Command::MetaCommand(MetaCommand::Format { format }) => {
                assert!(matches!(format, OutputFormat::JSON));
            }
            _ => panic!("Expected Format command for json"),
        }
    }

    #[test]
    fn test_parse_meta_command_format_error() {
        match parse_command("\\format") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_some());
            }
            _ => panic!("Expected Help command with error message"),
        }

        match parse_command("\\format invalid") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_some());
            }
            _ => panic!("Expected Help command for invalid format"),
        }
    }

    #[test]
    fn test_parse_meta_command_pager() {
        match parse_command("\\pager") {
            Command::MetaCommand(MetaCommand::Pager { command }) => {
                assert!(command.is_none());
            }
            _ => panic!("Expected Pager command"),
        }

        match parse_command("\\pager less") {
            Command::MetaCommand(MetaCommand::Pager { command }) => {
                assert_eq!(command, Some("less".to_string()));
            }
            _ => panic!("Expected Pager command with command"),
        }
    }

    #[test]
    fn test_parse_meta_command_timing() {
        assert!(matches!(
            parse_command("\\timing"),
            Command::MetaCommand(MetaCommand::Timing)
        ));
    }

    #[test]
    fn test_parse_meta_command_set() {
        match parse_command("\\set") {
            Command::MetaCommand(MetaCommand::ShowVariables) => {}
            _ => panic!("Expected ShowVariables command"),
        }

        match parse_command("\\set VAR") {
            Command::MetaCommand(MetaCommand::Set { name, value }) => {
                assert_eq!(name, "VAR");
                assert!(value.is_none());
            }
            _ => panic!("Expected Set command without value"),
        }

        match parse_command("\\set VAR value") {
            Command::MetaCommand(MetaCommand::Set { name, value }) => {
                assert_eq!(name, "VAR");
                assert_eq!(value, Some("value".to_string()));
            }
            _ => panic!("Expected Set command with value"),
        }

        match parse_command("\\set VAR some value with spaces") {
            Command::MetaCommand(MetaCommand::Set { name, value }) => {
                assert_eq!(name, "VAR");
                assert_eq!(value, Some("some value with spaces".to_string()));
            }
            _ => panic!("Expected Set command with spaced value"),
        }
    }

    #[test]
    fn test_parse_meta_command_unset() {
        match parse_command("\\unset VAR") {
            Command::MetaCommand(MetaCommand::Unset { name }) => {
                assert_eq!(name, "VAR");
            }
            _ => panic!("Expected Unset command"),
        }
    }

    #[test]
    fn test_parse_meta_command_execute_script() {
        match parse_command("\\i script.sql") {
            Command::MetaCommand(MetaCommand::ExecuteScript { path }) => {
                assert_eq!(path, "script.sql");
            }
            _ => panic!("Expected ExecuteScript command"),
        }
    }

    #[test]
    fn test_parse_meta_command_execute_script_raw() {
        match parse_command("\\ir script.sql") {
            Command::MetaCommand(MetaCommand::ExecuteScriptRaw { path }) => {
                assert_eq!(path, "script.sql");
            }
            _ => panic!("Expected ExecuteScriptRaw command"),
        }
    }

    #[test]
    fn test_parse_meta_command_output_redirect() {
        match parse_command("\\o") {
            Command::MetaCommand(MetaCommand::OutputRedirect { path }) => {
                assert!(path.is_none());
            }
            _ => panic!("Expected OutputRedirect command"),
        }

        match parse_command("\\o output.txt") {
            Command::MetaCommand(MetaCommand::OutputRedirect { path }) => {
                assert_eq!(path, Some("output.txt".to_string()));
            }
            _ => panic!("Expected OutputRedirect command with path"),
        }
    }

    #[test]
    fn test_parse_meta_command_shell_command() {
        match parse_command("\\! ls -la") {
            Command::MetaCommand(MetaCommand::ShellCommand { command }) => {
                assert_eq!(command, "ls -la");
            }
            _ => panic!("Expected ShellCommand"),
        }
    }

    #[test]
    fn test_parse_meta_command_version() {
        assert!(matches!(
            parse_command("\\version"),
            Command::MetaCommand(MetaCommand::Version)
        ));
    }

    #[test]
    fn test_parse_meta_command_copyright() {
        assert!(matches!(
            parse_command("\\copyright"),
            Command::MetaCommand(MetaCommand::Copyright)
        ));
    }

    #[test]
    fn test_parse_meta_command_begin() {
        assert!(matches!(
            parse_command("\\begin"),
            Command::MetaCommand(MetaCommand::Begin)
        ));
    }

    #[test]
    fn test_parse_meta_command_commit() {
        assert!(matches!(
            parse_command("\\commit"),
            Command::MetaCommand(MetaCommand::Commit)
        ));
    }

    #[test]
    fn test_parse_meta_command_rollback() {
        assert!(matches!(
            parse_command("\\rollback"),
            Command::MetaCommand(MetaCommand::Rollback)
        ));

        match parse_command("\\rollback to savepoint1") {
            Command::MetaCommand(MetaCommand::RollbackTo { name }) => {
                assert_eq!(name, "savepoint1");
            }
            _ => panic!("Expected RollbackTo command"),
        }
    }

    #[test]
    fn test_parse_meta_command_autocommit() {
        match parse_command("\\autocommit") {
            Command::MetaCommand(MetaCommand::Autocommit { value }) => {
                assert!(value.is_none());
            }
            _ => panic!("Expected Autocommit command"),
        }

        match parse_command("\\autocommit on") {
            Command::MetaCommand(MetaCommand::Autocommit { value }) => {
                assert_eq!(value, Some("on".to_string()));
            }
            _ => panic!("Expected Autocommit command with value"),
        }
    }

    #[test]
    fn test_parse_meta_command_isolation() {
        match parse_command("\\isolation") {
            Command::MetaCommand(MetaCommand::Isolation { level }) => {
                assert!(level.is_none());
            }
            _ => panic!("Expected Isolation command"),
        }

        match parse_command("\\isolation serializable") {
            Command::MetaCommand(MetaCommand::Isolation { level }) => {
                assert_eq!(level, Some("serializable".to_string()));
            }
            _ => panic!("Expected Isolation command with level"),
        }
    }

    #[test]
    fn test_parse_meta_command_savepoint() {
        match parse_command("\\savepoint sp1") {
            Command::MetaCommand(MetaCommand::Savepoint { name }) => {
                assert_eq!(name, "sp1");
            }
            _ => panic!("Expected Savepoint command"),
        }
    }

    #[test]
    fn test_parse_meta_command_release() {
        match parse_command("\\release sp1") {
            Command::MetaCommand(MetaCommand::ReleaseSavepoint { name }) => {
                assert_eq!(name, "sp1");
            }
            _ => panic!("Expected ReleaseSavepoint command"),
        }
    }

    #[test]
    fn test_parse_meta_command_txstatus() {
        assert!(matches!(
            parse_command("\\txstatus"),
            Command::MetaCommand(MetaCommand::TxStatus)
        ));
    }

    #[test]
    fn test_parse_meta_command_edit() {
        match parse_command("\\e") {
            Command::MetaCommand(MetaCommand::Edit { file, line }) => {
                assert!(file.is_none());
                assert!(line.is_none());
            }
            _ => panic!("Expected Edit command"),
        }

        match parse_command("\\edit file.sql") {
            Command::MetaCommand(MetaCommand::Edit { file, line }) => {
                assert_eq!(file, Some("file.sql".to_string()));
                assert!(line.is_none());
            }
            _ => panic!("Expected Edit command with file"),
        }

        match parse_command("\\e file.sql +10") {
            Command::MetaCommand(MetaCommand::Edit { file, line }) => {
                assert_eq!(file, Some("file.sql".to_string()));
                assert_eq!(line, Some(10));
            }
            _ => panic!("Expected Edit command with file and line"),
        }
    }

    #[test]
    fn test_parse_meta_command_print_buffer() {
        assert!(matches!(
            parse_command("\\p"),
            Command::MetaCommand(MetaCommand::PrintBuffer)
        ));
    }

    #[test]
    fn test_parse_meta_command_reset_buffer() {
        assert!(matches!(
            parse_command("\\r"),
            Command::MetaCommand(MetaCommand::ResetBuffer)
        ));
    }

    #[test]
    fn test_parse_meta_command_write_buffer() {
        match parse_command("\\w output.sql") {
            Command::MetaCommand(MetaCommand::WriteBuffer { file }) => {
                assert_eq!(file, "output.sql");
            }
            _ => panic!("Expected WriteBuffer command"),
        }
    }

    #[test]
    fn test_parse_meta_command_history() {
        match parse_command("\\history") {
            Command::MetaCommand(MetaCommand::History { action }) => match action {
                HistoryAction::Show { count } => assert_eq!(count, Some(20)),
                _ => panic!("Expected Show action"),
            },
            _ => panic!("Expected History command"),
        }

        match parse_command("\\history 50") {
            Command::MetaCommand(MetaCommand::History { action }) => match action {
                HistoryAction::Show { count } => assert_eq!(count, Some(50)),
                _ => panic!("Expected Show action with count"),
            },
            _ => panic!("Expected History command with count"),
        }

        match parse_command("\\history clear") {
            Command::MetaCommand(MetaCommand::History { action }) => match action {
                HistoryAction::Clear => {}
                _ => panic!("Expected Clear action"),
            },
            _ => panic!("Expected History command with clear"),
        }

        match parse_command("\\history search pattern") {
            Command::MetaCommand(MetaCommand::History { action }) => match action {
                HistoryAction::Search { pattern } => assert_eq!(pattern, "pattern"),
                _ => panic!("Expected Search action"),
            },
            _ => panic!("Expected History command with search"),
        }

        match parse_command("\\history exec 5") {
            Command::MetaCommand(MetaCommand::History { action }) => match action {
                HistoryAction::Exec { id } => assert_eq!(id, 5),
                _ => panic!("Expected Exec action"),
            },
            _ => panic!("Expected History command with exec"),
        }
    }

    #[test]
    fn test_parse_meta_command_if() {
        match parse_command("\\if VAR") {
            Command::MetaCommand(MetaCommand::If { condition }) => {
                assert_eq!(condition, "VAR");
            }
            _ => panic!("Expected If command"),
        }
    }

    #[test]
    fn test_parse_meta_command_elif() {
        match parse_command("\\elif VAR") {
            Command::MetaCommand(MetaCommand::Elif { condition }) => {
                assert_eq!(condition, "VAR");
            }
            _ => panic!("Expected Elif command"),
        }
    }

    #[test]
    fn test_parse_meta_command_else() {
        assert!(matches!(
            parse_command("\\else"),
            Command::MetaCommand(MetaCommand::Else)
        ));
    }

    #[test]
    fn test_parse_meta_command_endif() {
        assert!(matches!(
            parse_command("\\endif"),
            Command::MetaCommand(MetaCommand::EndIf)
        ));
    }

    #[test]
    fn test_parse_meta_command_explain() {
        match parse_command("\\explain MATCH (v) RETURN v") {
            Command::MetaCommand(MetaCommand::Explain {
                query,
                analyze,
                format,
            }) => {
                assert_eq!(query, "MATCH (v) RETURN v");
                assert!(!analyze);
                assert!(matches!(format, ExplainFormat::Text));
            }
            _ => panic!("Expected Explain command"),
        }

        match parse_command("\\explain analyze MATCH (v) RETURN v") {
            Command::MetaCommand(MetaCommand::Explain {
                query,
                analyze,
                format,
            }) => {
                assert_eq!(query, "MATCH (v) RETURN v");
                assert!(analyze);
            }
            _ => panic!("Expected Explain command with analyze"),
        }

        match parse_command("\\explain format=json MATCH (v) RETURN v") {
            Command::MetaCommand(MetaCommand::Explain { format, .. }) => {
                assert!(matches!(format, ExplainFormat::Json));
            }
            _ => panic!("Expected Explain command with json format"),
        }
    }

    #[test]
    fn test_parse_meta_command_profile() {
        match parse_command("\\profile MATCH (v) RETURN v") {
            Command::MetaCommand(MetaCommand::Profile { query }) => {
                assert_eq!(query, "MATCH (v) RETURN v");
            }
            _ => panic!("Expected Profile command"),
        }
    }

    #[test]
    fn test_parse_meta_command_import() {
        match parse_command("\\import csv data.csv tag Person") {
            Command::MetaCommand(MetaCommand::Import {
                file_path,
                target,
                batch_size,
                ..
            }) => {
                assert_eq!(file_path, "data.csv");
                assert!(matches!(target, ImportTarget::Vertex { .. }));
                assert!(batch_size.is_none());
            }
            _ => panic!("Expected Import command"),
        }

        match parse_command("\\import json data.json edge Friend 100") {
            Command::MetaCommand(MetaCommand::Import { batch_size, target, .. }) => {
                assert_eq!(batch_size, Some(100));
                assert!(matches!(target, ImportTarget::Edge { .. }));
            }
            _ => panic!("Expected Import command with batch size"),
        }
    }

    #[test]
    fn test_parse_meta_command_export() {
        match parse_command("\\export csv output.csv MATCH (v) RETURN v") {
            Command::MetaCommand(MetaCommand::Export {
                file_path,
                query,
                ..
            }) => {
                assert_eq!(file_path, "output.csv");
                assert_eq!(query, "MATCH (v) RETURN v");
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_parse_meta_command_copy() {
        match parse_command("\\copy Person from 'data.csv'") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_some());
                assert!(topic.as_ref().unwrap().contains("Usage"));
            }
            _ => panic!("Expected Help command with usage error"),
        }

        match parse_command("\\copy Person to 'data.csv'") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_some());
            }
            _ => panic!("Expected Help command with usage error"),
        }
    }

    #[test]
    fn test_parse_meta_command_x() {
        match parse_command("\\x") {
            Command::MetaCommand(MetaCommand::Format { format }) => {
                assert!(matches!(format, OutputFormat::Vertical));
            }
            _ => panic!("Expected Format Vertical command"),
        }
    }

    #[test]
    fn test_parse_meta_command_unknown() {
        match parse_command("\\unknown") {
            Command::MetaCommand(MetaCommand::Help { topic }) => {
                assert!(topic.is_some());
                assert!(topic.unwrap().contains("Unknown"));
            }
            _ => panic!("Expected Help command with error for unknown command"),
        }
    }

    #[test]
    fn test_copy_direction_enum() {
        let from = CopyDirection::From;
        let to = CopyDirection::To;

        assert!(matches!(from, CopyDirection::From));
        assert!(matches!(to, CopyDirection::To));
    }

    #[test]
    fn test_history_action_enum() {
        let show = HistoryAction::Show { count: Some(10) };
        let search = HistoryAction::Search {
            pattern: "test".to_string(),
        };
        let clear = HistoryAction::Clear;
        let exec = HistoryAction::Exec { id: 5 };

        assert!(matches!(show, HistoryAction::Show { .. }));
        assert!(matches!(search, HistoryAction::Search { .. }));
        assert!(matches!(clear, HistoryAction::Clear));
        assert!(matches!(exec, HistoryAction::Exec { .. }));
    }
}
