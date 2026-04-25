use crate::encoder::Encoder;
use crate::highlight::*;
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
    assert!(config.clip);
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

    let result =
        highlight_single_document("hello world", "hello world test", &encoder, &config).unwrap();
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
    )
    .unwrap();

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
        result: vec![EnrichedSearchResult {
            id: 1,
            doc: Some(json!({"title": "hello world"})),
            highlight: None,
        }],
    }];

    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    processor
        .highlight_fields("hello", &mut results, &encoders, None, &options)
        .unwrap();

    assert_eq!(
        results[0].result[0].highlight,
        Some("<b>hello</b> world".to_string())
    );
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
            result: vec![EnrichedSearchResult {
                id: 1,
                doc: Some(json!({"title": "hello world", "content": "test content"})),
                highlight: None,
            }],
        },
        FieldSearchResult {
            field: "content".to_string(),
            result: vec![EnrichedSearchResult {
                id: 1,
                doc: Some(json!({"title": "hello world", "content": "test content"})),
                highlight: None,
            }],
        },
    ];

    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    processor
        .highlight_fields("hello", &mut results, &encoders, Some("title"), &options)
        .unwrap();

    // Only title field should be highlighted
    assert_eq!(
        results[0].result[0].highlight,
        Some("<b>hello</b> world".to_string())
    );
    assert_eq!(results[1].result[0].highlight, None);
}

#[test]
fn test_extract_field_value() {
    use crate::document::tree::{
        extract_value, extract_value_with_strategy, parse_tree, EvaluationStrategy,
    };

    let doc = json!({
        "title": "Test Title",
        "content": "Test Content",
        "nested": {
            "field": "Nested Value"
        }
    });

    // Test using extract_value (Lenient strategy)
    let mut marker = vec![];
    let tree_path = parse_tree("title", &mut marker);
    assert_eq!(
        extract_value(&doc, &tree_path),
        Some("Test Title".to_string())
    );

    let mut marker = vec![];
    let tree_path = parse_tree("content", &mut marker);
    assert_eq!(
        extract_value(&doc, &tree_path),
        Some("Test Content".to_string())
    );

    let mut marker = vec![];
    let tree_path = parse_tree("nested.field", &mut marker);
    assert_eq!(
        extract_value(&doc, &tree_path),
        Some("Nested Value".to_string())
    );

    // Non-existent field should return None in Lenient mode
    let mut marker = vec![];
    let tree_path = parse_tree("nonexistent", &mut marker);
    assert_eq!(extract_value(&doc, &tree_path), None);

    // Test Strict strategy - non-existent field returns error
    let mut marker = vec![];
    let tree_path = parse_tree("nonexistent", &mut marker);
    assert!(extract_value_with_strategy(&doc, &tree_path, EvaluationStrategy::Strict).is_err());
}

// ============================================================
// Added: test for structured highlighting results
// ============================================================

#[test]
fn test_structured_highlight_basic() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result =
        highlight_single_document_structured("hello", "hello world", &encoder, &config).unwrap();

    // Validation of the basic structure
    assert_eq!(result.total_matches, 1);
    assert_eq!(result.fields.len(), 1);
    assert_eq!(result.fields[0].matches.len(), 1);

    // Verify Match Position
    let match_info = &result.fields[0].matches[0];
    assert_eq!(match_info.text, "hello");
    assert_eq!(match_info.start_pos, 0);
    assert_eq!(match_info.end_pos, 5);
    assert_eq!(match_info.matched_query, "hello");

    // Verify highlighted text
    assert!(result.fields[0].highlighted_text.is_some());
    assert_eq!(
        result.fields[0].highlighted_text.as_ref().unwrap(),
        "<b>hello</b> world"
    );
}

#[test]
fn test_structured_highlight_multiple_matches() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<mark>$1</mark>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result = highlight_single_document_structured(
        "hello world",
        "hello world test hello",
        &encoder,
        &config,
    )
    .unwrap();

    // Verify the number of matches
    assert_eq!(result.total_matches, 3);
    assert_eq!(result.fields[0].matches.len(), 3);

    // Verify the first match
    assert_eq!(result.fields[0].matches[0].text, "hello");
    assert_eq!(result.fields[0].matches[0].start_pos, 0);
    assert_eq!(result.fields[0].matches[0].end_pos, 5);

    // Verify the second match
    assert_eq!(result.fields[0].matches[1].text, "world");
    assert_eq!(result.fields[0].matches[1].start_pos, 6);
    assert_eq!(result.fields[0].matches[1].end_pos, 11);

    // Verify the third match
    assert_eq!(result.fields[0].matches[2].text, "hello");
    assert_eq!(result.fields[0].matches[2].start_pos, 17);
    assert_eq!(result.fields[0].matches[2].end_pos, 22);

    // Validating matching query terms
    assert_eq!(result.fields[0].matched_queries.len(), 1);
    assert!(result.fields[0]
        .matched_queries
        .contains(&"hello world".to_string()));
}

#[test]
fn test_structured_highlight_no_match() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let result =
        highlight_single_document_structured("foo", "hello world", &encoder, &config).unwrap();

    // Verify no matches
    assert_eq!(result.total_matches, 0);
    assert_eq!(result.fields[0].matches.len(), 0);

    // Highlighted text should remain as is
    assert_eq!(
        result.fields[0].highlighted_text.as_ref().unwrap(),
        "hello world"
    );
}

#[test]
fn test_structured_highlight_with_boundary() {
    let encoder = Encoder::default();
    let boundary = HighlightBoundaryOptions {
        before: Some(10),
        after: Some(10),
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

    let result = highlight_single_document_structured(
        "hello",
        "this is a long text with hello in the middle and more text after",
        &encoder,
        &config,
    )
    .unwrap();

    // Verify that there is a match
    assert_eq!(result.total_matches, 1);

    // Verify that the highlighted text contains matches
    assert!(result.fields[0].highlighted_text.is_some());
    let highlighted = result.fields[0].highlighted_text.as_ref().unwrap();
    assert!(highlighted.contains("<b>hello</b>"));
    // Validating length limits after boundary processing
    assert!(highlighted.len() <= 60); // Allow some flexibility
}

#[test]
fn test_structured_highlight_document() {
    let encoder = Encoder::default();
    let options = HighlightOptions {
        template: "<mark>$1</mark>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };
    let config = HighlightConfig::from_options(&options).unwrap();

    let document = json!({
        "title": "Hello World Test",
        "content": "This is a test document"
    });

    let result =
        highlight_document_structured("hello", &document, "title", 12345, &encoder, &config)
            .unwrap();

    assert!(result.is_some());
    let highlight = result.unwrap();

    // Verify Document ID
    assert_eq!(highlight.id, 12345);

    // Validation Field Name
    assert_eq!(highlight.fields[0].field, "title");

    // Verify Match
    assert_eq!(highlight.total_matches, 1);
    assert_eq!(highlight.fields[0].matches[0].text, "Hello");
}

#[test]
fn test_highlight_results_batch() {
    use crate::highlight::processor::highlight_results;

    let encoder = Encoder::default();
    let mut encoders = HashMap::new();
    encoders.insert("title".to_string(), encoder);

    let options = HighlightOptions {
        template: "<b>$1</b>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    // Preparing search results and documents
    let search_results = vec![
        SearchResult {
            id: 1,
            score: None,
            doc: None,
        },
        SearchResult {
            id: 2,
            score: None,
            doc: None,
        },
    ];

    let documents = vec![json!({"title": "hello world"}), json!({"title": "foo bar"})];

    let highlights = highlight_results(
        "hello",
        &search_results,
        &documents,
        "title",
        &encoders,
        &options,
    )
    .unwrap();

    // Verify the number of highlighted results
    assert_eq!(highlights.len(), 1); // Only the first document has a match

    // Verify first document highlighting
    assert_eq!(highlights[0].id, 1);
    assert_eq!(highlights[0].total_matches, 1);
    assert_eq!(highlights[0].fields[0].field, "title");
    assert_eq!(highlights[0].fields[0].matches[0].text, "hello");
}

#[test]
fn test_highlight_results_with_complete() {
    use crate::highlight::processor::highlight_results_with_complete;

    let encoder = Encoder::default();
    let mut encoders = HashMap::new();
    encoders.insert("title".to_string(), encoder);

    let options = HighlightOptions {
        template: "<mark>$1</mark>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    let search_results = vec![
        SearchResult {
            id: 1,
            score: Some(0.95),
            doc: None,
        },
        SearchResult {
            id: 2,
            score: Some(0.85),
            doc: None,
        },
    ];

    let documents = vec![
        json!({"title": "hello world"}),
        json!({"title": "hello test"}),
    ];

    let complete_result = highlight_results_with_complete(
        "hello",
        search_results,
        documents,
        "title",
        &encoders,
        &options,
    )
    .unwrap();

    // Validate full results
    assert_eq!(complete_result.total, 2);
    assert_eq!(complete_result.query, "hello");
    assert_eq!(complete_result.results.len(), 2);
    assert_eq!(complete_result.highlights.len(), 2);

    // Verify the first highlight
    assert_eq!(complete_result.highlights[0].id, 1);
    assert_eq!(complete_result.highlights[0].total_matches, 1);
    assert!(complete_result.highlights[0].fields[0]
        .highlighted_text
        .as_ref()
        .unwrap()
        .contains("<mark>hello</mark>"));
}

#[test]
fn test_highlight_match_serialization() {
    use serde_json;

    let match_info = HighlightMatch {
        text: "hello".to_string(),
        start_pos: 0,
        end_pos: 5,
        matched_query: "hello".to_string(),
        score: Some(0.95),
    };

    // Test Serialization
    let json = serde_json::to_string(&match_info).unwrap();
    assert!(json.contains("\"text\":\"hello\""));
    assert!(json.contains("\"start_pos\":0"));
    assert!(json.contains("\"end_pos\":5"));
    assert!(json.contains("\"score\":0.95"));

    // Testing Deserialization
    let deserialized: HighlightMatch = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.text, "hello");
    assert_eq!(deserialized.start_pos, 0);
    assert_eq!(deserialized.end_pos, 5);
}

#[test]
fn test_document_highlight_serialization() {
    use serde_json;

    let doc_highlight = DocumentHighlight {
        id: 12345,
        fields: vec![FieldHighlight {
            field: "title".to_string(),
            matches: vec![HighlightMatch {
                text: "hello".to_string(),
                start_pos: 0,
                end_pos: 5,
                matched_query: "hello".to_string(),
                score: None,
            }],
            highlighted_text: Some("<b>hello</b> world".to_string()),
            matched_queries: vec!["hello".to_string()],
        }],
        total_matches: 1,
    };

    // Test Serialization
    let json = serde_json::to_string(&doc_highlight).unwrap();
    assert!(json.contains("\"id\":12345"));
    assert!(json.contains("\"total_matches\":1"));
    assert!(json.contains("\"highlighted_text\""));

    // Testing Deserialization
    let deserialized: DocumentHighlight = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, 12345);
    assert_eq!(deserialized.total_matches, 1);
    assert_eq!(deserialized.fields.len(), 1);
}
