use crate::error::{Result, InversearchError, EncoderError};
use crate::r#type::EncoderOptions;

/// Validates encoder configuration options
pub struct EncoderValidator;

impl EncoderValidator {
    /// Validate encoder options for consistency and safety
    pub fn validate(options: &EncoderOptions) -> Result<()> {
        // Validate length constraints
        if let (Some(min), Some(max)) = (options.minlength, options.maxlength) {
            if min > max {
                return Err(InversearchError::Encoder(EncoderError::InvalidRegex("minlength cannot be greater than maxlength".to_string())));
            }
            if min == 0 {
                return Err(InversearchError::Encoder(EncoderError::InvalidRegex("minlength must be greater than 0".to_string())));
            }
        }

        // Validate filter configuration
        if let Some(filter) = &options.filter {
            if filter.len() > 10_000 {
                return Err(InversearchError::Encoder(EncoderError::Encoding("filter set too large (max 10,000 entries)".to_string())));
            }
        }

        // Validate mapper configuration
        if let Some(mapper) = &options.mapper {
            if mapper.len() > 1_000 {
                return Err(InversearchError::Encoder(EncoderError::Encoding("mapper too large (max 1,000 mappings)".to_string())));
            }
        }

        // Validate matcher configuration
        if let Some(matcher) = &options.matcher {
            if matcher.len() > 5_000 {
                return Err(InversearchError::Encoder(EncoderError::Encoding("matcher too large (max 5,000 patterns)".to_string())));
            }
        }

        // Validate replacer configuration
        if let Some(replacer) = &options.replacer {
            if replacer.len() > 100 {
                return Err(InversearchError::Encoder(EncoderError::Encoding("too many replacer patterns (max 100)".to_string())));
            }
            
            // Validate regex patterns
            for (pattern, _) in replacer {
                if pattern.len() > 1000 {
                    return Err(InversearchError::Encoder(EncoderError::InvalidRegex("replacer pattern too long (max 1000 chars)".to_string())));
                }
            }
        }

        // Validate stemmer configuration
        if let Some(stemmer) = &options.stemmer {
            if stemmer.len() > 1_000 {
                return Err(InversearchError::Encoder(EncoderError::Encoding("stemmer too large (max 1,000 rules)".to_string())));
            }
        }

        Ok(())
    }

    /// Validate transformer compatibility
    pub fn validate_transformers(
        prepare: Option<&dyn crate::encoder::TextTransformer>,
        finalize: Option<&dyn crate::encoder::TokenFinalizer>,
        _filter: Option<&dyn crate::encoder::TextFilter>,
    ) -> Result<()> {
        // Check for potential infinite loops or expensive operations
        // This is a basic check - could be expanded based on specific transformer implementations
        
        if prepare.is_some() && finalize.is_some() {
            // Warn about potential performance impact
            tracing::debug!("Both prepare and finalize transformers are set - this may impact performance");
        }

        Ok(())
    }

    /// Suggest optimizations based on configuration
    pub fn suggest_optimizations(options: &EncoderOptions) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Suggest caching for complex configurations
        let complexity_score = Self::calculate_complexity_score(options);
        if complexity_score > 50 {
            suggestions.push("Consider enabling cache for complex encoder configuration".to_string());
        }

        // Suggest deduplication for likely duplicate scenarios
        if options.dedupe == Some(false) && complexity_score > 30 {
            suggestions.push("Consider enabling deduplication for better performance".to_string());
        }

        // Suggest numeric splitting optimization
        if options.numeric != Some(false) {
            suggestions.push("Numeric splitting is enabled - consider disabling if not processing numeric data".to_string());
        }

        suggestions
    }

    fn calculate_complexity_score(options: &EncoderOptions) -> u32 {
        let mut score = 0;

        // Base complexity
        score += 10;

        // Add complexity for each transformation
        if options.stemmer.is_some() { score += 20; }
        if options.matcher.is_some() { score += 15; }
        if options.mapper.is_some() { score += 10; }
        if options.replacer.is_some() { score += 25; }
        if options.filter.is_some() { score += 5; }

        // Normalize adds complexity
        if options.normalize != Some(false) { score += 5; }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_length_constraints() {
        let mut options = EncoderOptions::default();
        options.minlength = Some(10);
        options.maxlength = Some(5);
        
        let result = EncoderValidator::validate(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_filter_size() {
        let mut options = EncoderOptions::default();
        let mut filter = Vec::new();
        for i in 0..10_001 {
            filter.push(format!("word{}", i));
        }
        options.filter = Some(filter);
        
        let result = EncoderValidator::validate(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_suggest_optimizations() {
        let options = EncoderOptions::default();
        let suggestions = EncoderValidator::suggest_optimizations(&options);
        assert!(!suggestions.is_empty());
    }
}