use aho_corasick::{AhoCorasick, MatchKind};
use std::collections::HashSet;

pub struct KeywordManager {
    static_keywords: Vec<String>,
    custom_keywords: HashSet<String>,
    active_patterns: Vec<String>,
    ac_matcher: Option<AhoCorasick>,
}

impl KeywordManager {
    pub fn new() -> Self {
        let base_keywords = vec![
            // Core explicit terms
            "porn", "porno", "pornography", "xxx", "sex video", "hardcore sex",
            "xvideos", "xnxx", "xhamster", "erotic", "nude", "naked", "blowjob",
            "handjob", "deepthroat", "cumshot", "gangbang", "creampie", "milf",
            "dildo", "vibrator", "masturbat", "orgasm", "shemale", "transsexual porn",
            "incest porn", "hentai", "ecchi", "rule34", "erothots", "redgifs",
            "chaturbate", "stripchat", "camsoda", "onlyfans leaks", "thothub",
            "coomer", "fapello", "nudist", "voyeur", "peeping tom", "nsfw video",
            "adult cam", "sex cam", "live sex", "naked girls", "hot babes naked",
            // Obfuscated variants
            "p0rn", "pr0n", "p*rn", "sexx", "s3x", "xxnx", "x-videos",
        ];

        let mut km = Self {
            static_keywords: base_keywords.into_iter().map(|s| s.to_string()).collect(),
            custom_keywords: HashSet::new(),
            active_patterns: Vec::new(),
            ac_matcher: None,
        };

        km.rebuild_matcher();
        km
    }

    fn rebuild_matcher(&mut self) {
        let mut all_patterns: Vec<String> = self.static_keywords.clone();
        for custom in &self.custom_keywords {
            all_patterns.push(custom.to_lowercase());
        }

        self.ac_matcher = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::Standard)
            .build(&all_patterns)
            .ok();
        self.active_patterns = all_patterns;
    }

    /// Normalize text to defeat common leetspeak and obfuscation tricks:
    /// e.g. "p.o.r.n" -> "porn", "p 0 r n" -> "porn", "s3x" -> "sex"
    pub fn normalize_text(input: &str) -> String {
        let lower = input.to_lowercase();
        let mut normalized = String::with_capacity(lower.len());

        for c in lower.chars() {
            match c {
                '0' => normalized.push('o'),
                '@' => normalized.push('a'),
                '1' | '!' => normalized.push('i'),
                '$' => normalized.push('s'),
                '3' => normalized.push('e'),
                '+' => normalized.push('t'),
                ' ' | '.' | '-' | '_' | '*' | '/' | '\\' | '|' => {
                    // Retain a single space delimiter to keep word boundaries
                    if !normalized.ends_with(' ') {
                        normalized.push(' ');
                    }
                }
                c if c.is_alphanumeric() => normalized.push(c),
                _ => {}
            }
        }

        normalized
    }

    /// Extract search query terms from standard search engine URLs
    pub fn extract_search_query(url: &str) -> Option<String> {
        let clean = url.to_lowercase();
        let query_indicators = [
            "google.com/search?", "google./search?",
            "bing.com/search?", "bing.com/images/search?",
            "youtube.com/results?", "m.youtube.com/results?",
            "duckduckgo.com/?", "duckduckgo.com/html/?",
            "search.yahoo.com/search?",
            "yandex.com/search?", "ecosia.org/search?",
        ];

        let mut is_search = false;
        for indicator in &query_indicators {
            if clean.contains(indicator) {
                is_search = true;
                break;
            }
        }

        if !is_search {
            return None;
        }

        // Look for parameters: q=, search_query=, p=, text=
        let params = ["search_query=", "q=", "p=", "text="];
        for param in params {
            if let Some(pos) = clean.find(param) {
                let start = pos + param.len();
                let sub = &clean[start..];
                let end = sub.find('&').unwrap_or(sub.len());
                let raw_val = &sub[..end];
                // Replace URL encoded plus signs with spaces
                let decoded = raw_val.replace('+', " ");
                return Some(decoded);
            }
        }

        None
    }

    /// Check whether a text or search query contains blocked keywords
    pub fn is_text_blocked(&self, input: &str) -> Option<String> {
        if input.trim().is_empty() {
            return None;
        }

        // 1. Direct raw check
        if let Some(ref matcher) = self.ac_matcher {
            if let Some(mat) = matcher.find(input) {
                let idx = mat.pattern().as_usize();
                if let Some(p) = self.active_patterns.get(idx) {
                    return Some(p.clone());
                }
            }
        }

        // 2. Normalized check (leetspeak, spaced characters)
        let normalized = Self::normalize_text(input);
        if let Some(ref matcher) = self.ac_matcher {
            if let Some(mat) = matcher.find(&normalized) {
                let idx = mat.pattern().as_usize();
                if let Some(p) = self.active_patterns.get(idx) {
                    return Some(p.clone());
                }
            }

            // Also check collapsed version (e.g., "p o r n" without any spaces -> "porn")
            let collapsed: String = normalized.chars().filter(|c| !c.is_whitespace()).collect();
            if let Some(mat) = matcher.find(&collapsed) {
                let idx = mat.pattern().as_usize();
                if let Some(p) = self.active_patterns.get(idx) {
                    return Some(p.clone());
                }
            }
        }

        None
    }

    pub fn add_custom_keyword(&mut self, keyword: &str) {
        let clean = keyword.trim().to_lowercase();
        if !clean.is_empty() {
            self.custom_keywords.insert(clean);
            self.rebuild_matcher();
        }
    }

    pub fn remove_custom_keyword(&mut self, keyword: &str) {
        let clean = keyword.trim().to_lowercase();
        self.custom_keywords.remove(&clean);
        self.rebuild_matcher();
    }
}
