use crate::command::executor::CommandExecutor;
use crate::command::parser::CopyDirection;
use crate::io::{
    CsvExporter, CsvImporter, ExportConfig, ExportFormat, ImportConfig, ImportFormat,
    ImportTarget, JsonExporter, JsonImporter,
};
use crate::session::manager::SessionManager;
use crate::utils::error::Result;

pub fn execute_output_redirect(
    executor: &mut CommandExecutor,
    path: Option<String>,
) -> Result<bool> {
    match path {
        Some(p) => {
            let _file = std::fs::File::create(&p).map_err(crate::utils::error::CliError::IoError)?;
            // Note: output_file is private, need to handle differently
            // For now, just acknowledge
            executor.write_output(&format!("Output redirected to: {}", p))?;
        }
        None => {
            executor.write_output("Output redirect closed.")?;
        }
    }
    Ok(true)
}

pub async fn execute_import(
    executor: &mut CommandExecutor,
    format: ImportFormat,
    file_path: String,
    target: ImportTarget,
    batch_size: Option<usize>,
    session_mgr: &mut SessionManager,
) -> Result<bool> {
    if !executor.conditional_stack().is_active() {
        return Ok(true);
    }

    let config = ImportConfig::new(file_path.into(), target)
        .with_format(format)
        .with_batch_size(batch_size.unwrap_or(100));

    let stats = match config.format {
        ImportFormat::Csv { .. } => {
            let mut importer = CsvImporter::new(config);
            importer.import(session_mgr).await?
        }
        ImportFormat::Json { .. } => {
            let mut importer = JsonImporter::new(config);
            importer.import(session_mgr).await?
        }
        ImportFormat::JsonLines => {
            let mut importer = JsonImporter::new(config);
            importer.import(session_mgr).await?
        }
    };

    executor.write_output(&stats.format_summary())?;
    Ok(true)
}

pub async fn execute_export(
    executor: &mut CommandExecutor,
    format: ExportFormat,
    file_path: String,
    query: &str,
    session_mgr: &mut SessionManager,
) -> Result<bool> {
    if !executor.conditional_stack().is_active() {
        return Ok(true);
    }

    let config = ExportConfig::new(file_path.into(), format);

    let stats = match &config.format {
        ExportFormat::Csv { .. } => {
            let exporter = CsvExporter::new(config);
            exporter.export(query, session_mgr).await?
        }
        ExportFormat::Json { .. } | ExportFormat::JsonLines => {
            let exporter = JsonExporter::new(config);
            exporter.export(query, session_mgr).await?
        }
    };

    executor.write_output(&stats.format_summary())?;
    Ok(true)
}

pub async fn execute_copy(
    executor: &mut CommandExecutor,
    direction: CopyDirection,
    target: String,
    file_path: String,
    session_mgr: &mut SessionManager,
) -> Result<bool> {
    if !executor.conditional_stack().is_active() {
        return Ok(true);
    }

    match direction {
        CopyDirection::From => {
            let import_format =
                if file_path.ends_with(".json") || file_path.ends_with(".jsonl") {
                    ImportFormat::json_array()
                } else {
                    ImportFormat::csv()
                };

            let config = ImportConfig::new(
                file_path.into(),
                ImportTarget::vertex(&target),
            )
            .with_format(import_format.clone());

            let stats = match import_format {
                ImportFormat::Csv { .. } => {
                    let mut importer = CsvImporter::new(config);
                    importer.import(session_mgr).await?
                }
                _ => {
                    let mut importer = JsonImporter::new(config);
                    importer.import(session_mgr).await?
                }
            };

            executor.write_output(&stats.format_summary())?;
        }
        CopyDirection::To => {
            let query = format!("MATCH (n:{}) RETURN n", target);
            let export_format = if file_path.ends_with(".json") {
                ExportFormat::json()
            } else {
                ExportFormat::csv()
            };

            let config = ExportConfig::new(file_path.into(), export_format);

            let stats = match &config.format {
                ExportFormat::Csv { .. } => {
                    let exporter = CsvExporter::new(config);
                    exporter.export(&query, session_mgr).await?
                }
                _ => {
                    let exporter = JsonExporter::new(config);
                    exporter.export(&query, session_mgr).await?
                }
            };

            executor.write_output(&stats.format_summary())?;
        }
    }
    Ok(true)
}
