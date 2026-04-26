use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

use anyhow::Result;

use crate::io::{ImportConfig, ImportError, ImportStats, ImportTarget};
use crate::session::manager::SessionManager;

pub struct JsonImporter {
    config: ImportConfig,
    batch_buffer: Vec<String>,
    start_time: Instant,
}

impl JsonImporter {
    pub fn new(config: ImportConfig) -> Self {
        Self {
            config,
            batch_buffer: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub async fn import(&mut self, session: &mut SessionManager) -> Result<ImportStats> {
        let content = fs::read_to_string(&self.config.file_path)?;
        let mut stats = ImportStats::new();

        if self.config.format.is_array_mode() {
            let items: Vec<serde_json::Value> = serde_json::from_str(&content)?;
            for (idx, item) in items.iter().enumerate() {
                match self.process_json_item(item, session).await {
                    Ok(_) => stats.success_rows += 1,
                    Err(e) => {
                        stats.failed_rows += 1;
                        stats
                            .errors
                            .push(ImportError::new(idx, item.to_string(), e.to_string()));
                    }
                }
                stats.total_rows += 1;
            }
        } else {
            let file = File::open(&self.config.file_path)?;
            let reader = BufReader::new(file);

            for (idx, line) in reader.lines().enumerate() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                let item: serde_json::Value = serde_json::from_str(&line)?;
                match self.process_json_item(&item, session).await {
                    Ok(_) => stats.success_rows += 1,
                    Err(e) => {
                        stats.failed_rows += 1;
                        stats
                            .errors
                            .push(ImportError::new(idx, line.clone(), e.to_string()));
                    }
                }
                stats.total_rows += 1;
            }
        }

        self.flush_batch(session).await?;
        stats.duration_ms = self.start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }

    async fn process_json_item(
        &mut self,
        json: &serde_json::Value,
        session: &mut SessionManager,
    ) -> Result<()> {
        let query = self.build_insert_from_json(json)?;
        self.batch_buffer.push(query);

        if self.batch_buffer.len() >= self.config.batch_size {
            self.flush_batch(session).await?;
        }

        Ok(())
    }

    fn build_insert_from_json(&self, json: &serde_json::Value) -> Result<String> {
        let obj = json
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected JSON object"))?;

        match &self.config.target_type {
            ImportTarget::Vertex { tag } => {
                let vid = obj
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing _id field"))?;

                let mut fields = Vec::new();
                let mut values = Vec::new();

                for (key, value) in obj.iter() {
                    if key == "_id" {
                        continue;
                    }
                    fields.push(self.config.map_field_name(key));
                    values.push(json_value_to_gql(value));
                }

                Ok(format!(
                    "INSERT VERTEX {} ({}) VALUES \"{}\":({})",
                    tag,
                    fields.join(", "),
                    vid,
                    values.join(", ")
                ))
            }
            ImportTarget::Edge { edge_type } => {
                let src = obj
                    .get("_src")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing _src field"))?;
                let dst = obj
                    .get("_dst")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing _dst field"))?;

                let mut fields = Vec::new();
                let mut values = Vec::new();

                for (key, value) in obj.iter() {
                    if key == "_src" || key == "_dst" {
                        continue;
                    }
                    fields.push(self.config.map_field_name(key));
                    values.push(json_value_to_gql(value));
                }

                Ok(format!(
                    "INSERT EDGE {} ({}) VALUES \"{}\"->\"{}\":({})",
                    edge_type,
                    fields.join(", "),
                    src,
                    dst,
                    values.join(", ")
                ))
            }
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

fn json_value_to_gql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_value_to_gql).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(_) => {
            format!("\"{}\"", serde_json::to_string(value).unwrap_or_default())
        }
    }
}

pub struct JsonExporter {
    config: crate::io::ExportConfig,
    start_time: Instant,
}

impl JsonExporter {
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

        match &self.config.format {
            crate::io::ExportFormat::Json {
                pretty,
                array_wrapper,
            } => {
                if *array_wrapper {
                    writer.write_all(b"[\n")?;
                }

                for (idx, row) in result.rows.iter().enumerate() {
                    let obj = self.row_to_json_object(&result.columns, row);

                    let json_str = if *pretty {
                        serde_json::to_string_pretty(&obj)?
                    } else {
                        serde_json::to_string(&obj)?
                    };

                    if *array_wrapper && idx > 0 {
                        writer.write_all(b",\n")?;
                    }
                    writer.write_all(json_str.as_bytes())?;

                    stats.total_rows += 1;
                }

                if *array_wrapper {
                    writer.write_all(b"\n]")?;
                }
            }
            crate::io::ExportFormat::JsonLines => {
                for row in &result.rows {
                    let obj = self.row_to_json_object(&result.columns, row);
                    let json_str = serde_json::to_string(&obj)?;
                    writeln!(writer, "{}", json_str)?;
                    stats.total_rows += 1;
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid format for JSON exporter")),
        }

        writer.flush()?;
        stats.bytes_written = writer.get_ref().metadata()?.len();
        stats.duration_ms = self.start_time.elapsed().as_millis() as u64;

        Ok(stats)
    }

    fn row_to_json_object(
        &self,
        columns: &[String],
        row: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for col in columns {
            let value = row.get(col).cloned().unwrap_or(serde_json::Value::Null);
            obj.insert(col.clone(), value);
        }
        serde_json::Value::Object(obj)
    }
}
