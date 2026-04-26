use crate::command::parser::types::{CopyDirection, MetaCommand};
use crate::io::{ExportFormat, ImportFormat, ImportTarget};

pub fn parse_execute_script(arg: &str) -> Result<MetaCommand, String> {
    if arg.is_empty() {
        Err("Usage: \\i <file_path>".to_string())
    } else {
        Ok(MetaCommand::ExecuteScript {
            path: arg.to_string(),
        })
    }
}

pub fn parse_execute_script_raw(arg: &str) -> Result<MetaCommand, String> {
    if arg.is_empty() {
        Err("Usage: \\ir <file_path>".to_string())
    } else {
        Ok(MetaCommand::ExecuteScriptRaw {
            path: arg.to_string(),
        })
    }
}

pub fn parse_output_redirect(arg: &str) -> Result<MetaCommand, String> {
    let path = if arg.is_empty() {
        None
    } else {
        Some(arg.to_string())
    };
    Ok(MetaCommand::OutputRedirect { path })
}

pub fn parse_import(arg: &str) -> Result<MetaCommand, String> {
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

pub fn parse_export(arg: &str) -> Result<MetaCommand, String> {
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

pub fn parse_copy(arg: &str) -> Result<MetaCommand, String> {
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
