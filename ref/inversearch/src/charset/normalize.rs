use crate::r#type::EncoderOptions;

pub fn get_charset_normalize() -> EncoderOptions {
    EncoderOptions::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_normalize() {
        let options = get_charset_normalize();
        assert_eq!(options.normalize, Some(true)); // Default is Some(true)
        assert_eq!(options.dedupe, Some(true)); // Default is Some(true)
    }
}