use unicode_normalization::UnicodeNormalization;

pub struct UnicodeNormalizer;

impl UnicodeNormalizer {
    pub fn new() -> Self {
        UnicodeNormalizer
    }
    
    pub fn normalize(&self, text: &str) -> String {
        text.nfc().collect()
    }
}

pub fn normalize_unicode_path(path: &str) -> String {
    path.nfc().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalization() {
        let normalizer = UnicodeNormalizer::new();
        let input = "café";
        let normalized = normalizer.normalize(input);
        assert!(!normalized.is_empty());
    }
}
