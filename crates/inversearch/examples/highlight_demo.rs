use inversearch_service::{
    highlight::*, encoder::Encoder, 
};
use serde_json::json;
use std::collections::HashMap;

fn main() {
    // 创建高亮配置
    let highlight_options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(10),
            after: Some(10),
            total: Some(100),
        }),
        clip: Some(true),
        merge: Some(true),
        ellipsis: Some(HighlightEllipsisOptions {
            template: "[$1]".to_string(),
            pattern: Some("...".to_string()),
        }),
    };

    // 创建编码器
    let encoder = Encoder::default();

    // 示例1: 简单的文档高亮 (使用英文示例避免Unicode问题)
    println!("=== 简单文档高亮示例 ===");
    let document = json!({
        "title": "Rust Programming Language Introduction",
        "content": "Rust is a systems programming language focused on safety, speed, and concurrency."
    });

    let config = HighlightConfig::from_options(&highlight_options).unwrap();
    let result = highlight_document("Rust", &document, "content", &encoder, &config).unwrap();
    
    if let Some(highlighted) = result {
        println!("Original content: {}", document["content"]);
        println!("Highlighted result: {}", highlighted);
    }

    // 示例2: 多字段搜索结果高亮
    println!("\n=== 多字段搜索结果高亮示例 ===");
    
    let mut results = vec![
        FieldSearchResult {
            field: "title".to_string(),
            result: vec![
                EnrichedSearchResult {
                    id: 1,
                    doc: Some(json!({
                        "title": "Rust Programming Language",
                        "content": "Rust provides memory safety guarantees without garbage collector."
                    })),
                    highlight: None,
                },
                EnrichedSearchResult {
                    id: 2,
                    doc: Some(json!({
                        "title": "JavaScript vs Rust Comparison",
                        "content": "JavaScript and Rust have different memory management approaches."
                    })),
                    highlight: None,
                },
            ],
        },
        FieldSearchResult {
            field: "content".to_string(),
            result: vec![
                EnrichedSearchResult {
                    id: 1,
                    doc: Some(json!({
                        "title": "Rust Programming Language",
                        "content": "Rust provides memory safety guarantees without garbage collector."
                    })),
                    highlight: None,
                },
            ],
        },
    ];

    // 创建编码器映射
    let mut encoders = HashMap::new();
    encoders.insert("title".to_string(), encoder.clone());
    encoders.insert("content".to_string(), encoder.clone());

    // 使用处理器进行高亮
    let mut processor = HighlightProcessor::new();
    processor.highlight_fields("Rust", &mut results, &encoders, None, &highlight_options).unwrap();

    // 显示结果
    for field_result in &results {
        println!("\nField: {}", field_result.field);
        for search_result in &field_result.result {
            if let Some(highlight) = &search_result.highlight {
                println!("  Document {} highlight: {}", search_result.id, highlight);
            }
        }
    }

    // 示例3: 使用pluck模式只高亮特定字段
    println!("\n=== Pluck模式示例 ===");
    
    let mut pluck_results = vec![
        FieldSearchResult {
            field: "title".to_string(),
            result: vec![
                EnrichedSearchResult {
                    id: 1,
                    doc: Some(json!({
                        "title": "Rust Programming Language",
                        "content": "Rust provides memory safety guarantees."
                    })),
                    highlight: None,
                },
            ],
        },
        FieldSearchResult {
            field: "content".to_string(),
            result: vec![
                EnrichedSearchResult {
                    id: 1,
                    doc: Some(json!({
                        "title": "Rust Programming Language",
                        "content": "Rust provides memory safety guarantees."
                    })),
                    highlight: None,
                },
            ],
        },
    ];

    processor.highlight_fields("Rust", &mut pluck_results, &encoders, Some("title"), &highlight_options).unwrap();

    println!("Only highlight title field:");
    for field_result in &pluck_results {
        for search_result in &field_result.result {
            if let Some(highlight) = &search_result.highlight {
                println!("  Field {} highlight: {}", field_result.field, highlight);
            } else if field_result.field != "title" {
                println!("  Field {} not highlighted (as expected)", field_result.field);
            }
        }
    }
}