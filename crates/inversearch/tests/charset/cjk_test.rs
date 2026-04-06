//! CJK (中日韩) 字符集测试
//!
//! 测试范围：
//! - 中文字符
//! - 日文字符
//! - 韩文字符
//! - 混合 CJK 文本

use inversearch_service::search::search;

use crate::common::{
    create_empty_index, basic_search_options,
};

/// 测试基本中文字符
#[test]
fn test_basic_chinese() {
    let mut index = create_empty_index();

    index.add(1, "这是一个中文测试", false).unwrap();
    index.add(2, "搜索引擎很重要", false).unwrap();

    let options = basic_search_options("中文");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.results.contains(&1), "Expected results to contain document 1");
}

/// 测试简体中文
#[test]
fn test_simplified_chinese() {
    let mut index = create_empty_index();

    index.add(1, "简体中文测试", false).unwrap();
    index.add(2, "编程语言", false).unwrap();

    let options = basic_search_options("编程");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试繁体中文
#[test]
fn test_traditional_chinese() {
    let mut index = create_empty_index();

    index.add(1, "繁體中文測試", false).unwrap();
    index.add(2, "程式語言", false).unwrap();

    let options = basic_search_options("程式");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试日文平假名
#[test]
fn test_japanese_hiragana() {
    let mut index = create_empty_index();

    index.add(1, "ひらがなテスト", false).unwrap();
    index.add(2, "これは日本語です", false).unwrap();

    let options = basic_search_options("ひらがな");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试日文片假名
#[test]
fn test_japanese_katakana() {
    let mut index = create_empty_index();

    index.add(1, "カタカナテスト", false).unwrap();
    index.add(2, "プログラミング言語", false).unwrap();

    let options = basic_search_options("プログラミング");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试日文汉字
#[test]
fn test_japanese_kanji() {
    let mut index = create_empty_index();

    index.add(1, "日本語の漢字テスト", false).unwrap();
    index.add(2, "検索エンジン", false).unwrap();

    let options = basic_search_options("検索");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试韩文
#[test]
fn test_korean() {
    let mut index = create_empty_index();

    index.add(1, "한국어 테스트", false).unwrap();
    index.add(2, "프로그래밍 언어", false).unwrap();

    let options = basic_search_options("프로그래밍");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试 CJK 数字混合
#[test]
fn test_cjk_with_numbers() {
    let mut index = create_empty_index();

    index.add(1, "第1章 介绍", false).unwrap();
    index.add(2, "Version 2.0 版本", false).unwrap();

    let options = basic_search_options("第1章");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试长中文文本
#[test]
fn test_long_chinese_text() {
    let mut index = create_empty_index();

    let long_text = "这是一个很长的中文文本，用于测试搜索引擎对长文本的处理能力。\
                     搜索引擎需要能够正确地索引和搜索长文本中的内容。\
                     这个测试将验证系统是否能够处理包含多个句子的中文文本。";

    index.add(1, long_text, false).unwrap();

    let options = basic_search_options("搜索引擎");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}
