use crate::r#type::EncoderOptions;

pub fn get_charset_cjk() -> EncoderOptions {
    EncoderOptions {
        split: Some("".to_string()),
        rtl: Some(false),
        dedupe: Some(true),
        numeric: Some(true),
        normalize: Some(true),
        prepare: None,
        finalize: None,
        filter: None,
        matcher: None,
        mapper: None,
        stemmer: None,
        replacer: None,
        minlength: Some(1),
        maxlength: Some(1024),
        cache: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_cjk() {
        let options = get_charset_cjk();
        assert_eq!(options.split, Some("".to_string()));
    }
}