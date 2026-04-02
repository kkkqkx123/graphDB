use crate::highlight::*;
use crate::encoder::Encoder;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_highlight_config_from_options() {
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: Some(true),
        ellipsis: Some(HighlightEllipsisOptions {
            template: "[$1]".to_string(),
            pattern: Some("...".to_string()),
        }),
    };

    let config = HighlightConfig::from_options(&options).unwrap();
    assert_eq!(config.markup_open, "<b>");
    assert_eq!(config.markup_close, "</b>");
    assert_eq!(config.clip, true);
    assert_eq!(config.merge, Some("</b> <b>".to_string()));
    assert_eq!(config.ellipsis, "[...]");
}

#[test]
fn test_basic_highlight() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result = highlight_single_document("hello", "hello world", &encoder, &config).unwrap();
    assert_eq!(result, "<b>hello</b> world");
}

#[test]
fn test_multiple_matches() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result = highlight_single_document("hello world", "hello world test", &encoder, &config).unwrap();
    assert_eq!(result, "<b>hello</b> <b>world</b> test");
}

#[test]
fn test_no_match() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result = highlight_single_document("foo", "hello world", &encoder, &config).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_boundary_simple() {
    let encoder = Encoder::default();
    let boundary = HighlightBoundaryOptions {
        before: Some(5),
        after: Some(5),
        total: Some(50),
    };
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: Some(boundary),
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result = highlight_single_document(
        "hello",
        "this is a long text with hello in the middle and more text after",
        &encoder,
        &config,
    ).unwrap();
    
    assert!(result.contains("<b>hello</b>"));
    assert!(result.len() <= 50);
}

#[test]
fn test_processor_single_field() {
    let mut processor = HighlightProcessor::new();
    let encoder = Encoder::default();
    let mut encoders = HashMap::new();
    encoders.insert("title".to_string(), encoder.clone());

    let mut results = vec![FieldSearchResult {
        field: "title".to_string(),
        result: vec![
            EnrichedSearchResult {
                id: 1,
                doc: Some(json!({"title": "hello world"})),
                highlight: None,
            },
        ],
    }];

    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    processor.highlight_fields("hello", &mut results, &encoders, None, &options).unwrap();
    
    assert_eq!(results[0].result[0].highlight, Some("<b>hello</b> world".to_string()));
}

#[test]
fn test_processor_pluck() {
    let mut processor = HighlightProcessor::new();
    let encoder = Encoder::default();
    let mut encoders = HashMap::new();
    encoders.insert("title".to_string(), encoder.clone());
    encoders.insert("content".to_string(), encoder.clone());

    let mut results = vec![
        FieldSearchResult {
            field: "title".to_string(),
            result: vec![
                EnrichedSearchResult {
                    id: 1,
                    doc: Some(json!({"title": "hello world", "content": "test content"})),
                    highlight: None,
                },
            ],
        },
        FieldSearchResult {
            field: "content".to_string(),
            result: vec![
                EnrichedSearchResult {
                    id: 1,
                    doc: Some(json!({"title": "hello world", "content": "test content"})),
                    highlight: None,
                },
            ],
        },
    ];

    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    processor.highlight_fields("hello", &mut results, &encoders, Some("title"), &options).unwrap();
    
    // Only title field should be highlighted
    assert_eq!(results[0].result[0].highlight, Some("<b>hello</b> world".to_string()));
    assert_eq!(results[1].result[0].highlight, None);
}

#[test]
fn test_parse_simple() {
    let doc = json!({
        "title": "Test Title",
        "content": "Test Content",
        "nested": {
            "field": "Nested Value"
        }
    });

    assert_eq!(crate::common::parse_simple(&doc, "title").unwrap(), "Test Title");
    assert_eq!(crate::common::parse_simple(&doc, "content").unwrap(), "Test Content");
    assert_eq!(crate::common::parse_simple(&doc, "nested.field").unwrap(), "Nested Value");
    assert_eq!(crate::common::parse_simple(&doc, "nonexistent").unwrap(), "");
}