use crate::r#type::EncoderOptions;

pub fn get_charset_exact() -> EncoderOptions {
    EncoderOptions {
        normalize: Some(false),
        numeric: Some(false),
        dedupe: Some(false),
        rtl: Some(false),
        split: None,
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
    fn test_charset_exact() {
        let options = get_charset_exact();
        assert_eq!(options.normalize, Some(false));
        assert_eq!(options.numeric, Some(false));
        assert_eq!(options.dedupe, Some(false));
    }
}