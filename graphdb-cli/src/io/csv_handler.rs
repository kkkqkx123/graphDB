use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::time::Instant;

use anyhow::Result;
use csv::ReaderBuilder;

use crate::io::{ErrorHandling, ImportConfig, ImportError, ImportStats, ImportTarget};
use crate::session::manager::SessionManager;

pub struct CsvImporter {
    config: ImportConfig,
    batch_buffer: Vec<String>,
    start_time: Instant,
}

impl CsvImporter {
    pub fn new(config: ImportConfig) -> Self {
        Self {
            config,
            batch_buffer: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub async fn import(&mut self, session: &mut SessionManager) -> Result<ImportStats> {
        let file = File::open(&self.config.file_path)?;
        let reader = BufReader::new(file);

        let mut csv_reader = ReaderBuilder::new()
            .delimiter(self.config.format.delimiter() as u8)
            .has_headers(self.config.format.has_header())
            .from_reader(reader);

        let headers = csv_reader.headers()?.clone();
        let mut stats = ImportStats::new();

        for (idx, result) in csv_reader.records().enumerate() {
            if idx < self.config.skip_rows {
                stats.skipped_rows += 1;
                continue;
            }

            match result {
                Ok(record) => match self.process_record(&headers, &record, session).await {
                    Ok(_) => stats.success_rows += 1,
                    Err(e) => {
                        stats.failed_rows += 1;
                        stats.errors.push(ImportError::new(
                            idx,
                            record.iter().collect::<Vec<_>>().join(","),
                            e.to_string(),
                        ));

                        if matches!(self.config.on_error, ErrorHandling::Stop) {
                            break;
                        }
                    }
                },
                Err(e) => {
                    stats.failed_rows += 1;
                    if matches!(self.config.on_error, ErrorHandling::Stop) {
                        return Err(e.into());
                    }
                }
            }

            stats.total_rows += 1;
        }

        self.flush_batch(session).await?;
        stats.duration_ms = self.start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }

    async fn process_record(
        &mut self,
        headers: &csv::StringRecord,
        record: &csv::StringRecord,
        session: &mut SessionManager,
    ) -> Result<()> {
        let query = self.build_insert_query(headers, record)?;
        self.batch_buffer.push(query);

        if self.batch_buffer.len() >= self.config.batch_size {
            self.flush_batch(session).await?;
        }

        Ok(())
    }

    fn build_insert_query(
        &self,
        headers: &csv::StringRecord,
        record: &csv::StringRecord,
    ) -> Result<String> {
        match &self.config.target_type {
            ImportTarget::Vertex { tag } => {
                let fields: Vec<String> = headers
                    .iter()
                    .map(|h| self.config.map_field_name(h))
                    .collect();
                let values: Vec<String> =
                    record.iter().map(|v| self.config.format_value(v)).collect();

                let vid = self.generate_vid(record)?;

                Ok(format!(
                    "INSERT VERTEX {} ({}) VALUES \"{}\":({})",
                    tag,
                    fields.join(", "),
                    vid,
                    values.join(", ")
                ))
            }
            ImportTarget::Edge { edge_type } => {
                let src_vid = record
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Missing source VID"))?;
                let dst_vid = record
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Missing destination VID"))?;

                let fields: Vec<String> = headers
                    .iter()
                    .skip(2)
                    .map(|h| self.config.map_field_name(h))
                    .collect();
                let values: Vec<String> = record
                    .iter()
                    .skip(2)
                    .map(|v| self.config.format_value(v))
                    .collect();

                Ok(format!(
                    "INSERT EDGE {} ({}) VALUES \"{}\"->\"{}\":({})",
                    edge_type,
                    fields.join(", "),
                    src_vid,
                    dst_vid,
                    values.join(", ")
                ))
            }
        }
    }

    fn generate_vid(&self, record: &csv::StringRecord) -> Result<String> {
        if let Some(vid) = record.get(0) {
            Ok(vid.to_string())
        } else {
            Ok(format!("vid_{}", uuid::Uuid::new_v4()))
        }
    }

    async fn flush_batch(&mut self, session: &mut SessionManager) -> Result<()> {
        if self.batch_buffer.is_empty() {
            return Ok(());
        }

        let queries: Vec<&str> = self.batch_buffer.iter().map(|s| s.as_str()).collect();
        let combined = queries.join("; ");

        session.execute_query(&combined).await?;
        self.batch_buffer.clear();

        Ok(())
    }
}

pub struct CsvExporter {
    config: crate::io::ExportConfig,
    start_time: Instant,
}

impl CsvExporter {
    pub fn new(config: crate::io::ExportConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
        }
    }

    pub async fn export(
        &self,
        query: &str,
        session: &mut SessionManager,
    ) -> Result<crate::io::ExportStats> {
        let result = session.execute_query(query).await?;
        let mut stats = crate::io::ExportStats::new();

        let file = File::create(&self.config.file_path)?;
        let mut writer = BufWriter::new(file);

        let delimiter = self.config.format.delimiter();

        if self.config.include_header {
            let header = result.columns.join(&delimiter.to_string());
            writeln!(writer, "{}", header)?;
        }

        for row in &result.rows {
            let values: Vec<String> = result
                .columns
                .iter()
                .map(|col| {
                    row.get(col)
                        .map(|v| self.format_csv_value(v))
                        .unwrap_or_default()
                })
                .collect();

            writeln!(writer, "{}", values.join(&delimiter.to_string()))?;
            stats.total_rows += 1;
        }

        writer.flush()?;
        stats.bytes_written = writer.get_ref().metadata()?.len();
        stats.duration_ms = self.start_time.elapsed().as_millis() as u64;

        Ok(stats)
    }

    fn format_csv_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => String::new(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => {
                if s.contains(',') || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('\"', "\"\""))
                } else {
                    s.clone()
                }
            }
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                serde_json::to_string(value).unwrap_or_default()
            }
        }
    }
}
