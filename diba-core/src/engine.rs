use crate::blocklist::BlocklistManager;
use crate::keywords::KeywordManager;
use crate::security::SecurityManager;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDecision {
    pub is_blocked: bool,
    pub reason: Option<String>,
    pub matched_term: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_scanned: u64,
    pub total_blocked: u64,
    pub rules_count: usize,
    pub is_active: bool,
    pub has_pin: bool,
}

pub struct DibaEngine {
    blocklist: BlocklistManager,
    keywords: KeywordManager,
    security: SecurityManager,
    total_scanned: u64,
    total_blocked: u64,
}

impl DibaEngine {
    pub fn new() -> Self {
        Self {
            blocklist: BlocklistManager::new(),
            keywords: KeywordManager::new(),
            security: SecurityManager::new(),
            total_scanned: 0,
            total_blocked: 0,
        }
    }

    pub fn evaluate_url(&mut self, url: &str) -> BlockDecision {
        if !self.security.is_active() {
            return BlockDecision {
                is_blocked: false,
                reason: None,
                matched_term: None,
            };
        }

        self.total_scanned += 1;
        let host = BlocklistManager::extract_host(url);

        // 0. Whitelist has highest priority
        if self.blocklist.is_whitelisted(&host) {
            return BlockDecision {
                is_blocked: false,
                reason: None,
                matched_term: None,
            };
        }

        // 1. Check if the domain itself is in adult blocklist
        if self.blocklist.is_domain_blocked(url) {
            self.total_blocked += 1;
            return BlockDecision {
                is_blocked: true,
                reason: Some("Adult Domain Blocked".to_string()),
                matched_term: Some(host),
            };
        }

        // 2. Check if this is a search engine URL and scan the query terms
        if let Some(query) = KeywordManager::extract_search_query(url) {
            if let Some(matched) = self.keywords.is_text_blocked(&query) {
                self.total_blocked += 1;
                return BlockDecision {
                    is_blocked: true,
                    reason: Some("Prohibited Search Query".to_string()),
                    matched_term: Some(matched),
                };
            }
        }

        // 3. Check for keywords embedded in URL path
        if let Some(matched) = self.keywords.is_text_blocked(url) {
            self.total_blocked += 1;
            return BlockDecision {
                is_blocked: true,
                reason: Some("Explicit URL Keyword".to_string()),
                matched_term: Some(matched),
            };
        }

        BlockDecision {
            is_blocked: false,
            reason: None,
            matched_term: None,
        }
    }

    pub fn evaluate_text(&mut self, text: &str) -> BlockDecision {
        if !self.security.is_active() {
            return BlockDecision {
                is_blocked: false,
                reason: None,
                matched_term: None,
            };
        }

        self.total_scanned += 1;

        if let Some(matched) = self.keywords.is_text_blocked(text) {
            self.total_blocked += 1;
            return BlockDecision {
                is_blocked: true,
                reason: Some("Explicit On-Screen Text".to_string()),
                matched_term: Some(matched),
            };
        }

        BlockDecision {
            is_blocked: false,
            reason: None,
            matched_term: None,
        }
    }

    pub fn add_blocked_domain(&mut self, domain: &str) {
        self.blocklist.add_custom_blacklist(domain);
    }

    pub fn add_whitelisted_domain(&mut self, domain: &str) {
        self.blocklist.add_custom_whitelist(domain);
    }

    pub fn add_blocked_keyword(&mut self, keyword: &str) {
        self.keywords.add_custom_keyword(keyword);
    }

    pub fn set_pin(&mut self, pin: &str) -> bool {
        self.security.set_pin(pin)
    }

    pub fn verify_pin(&self, pin: &str) -> bool {
        self.security.verify_pin(pin)
    }

    pub fn set_protection_active(&mut self, active: bool) {
        self.security.set_active(active);
    }

    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            total_scanned: self.total_scanned,
            total_blocked: self.total_blocked,
            rules_count: self.blocklist.get_rules_count(),
            is_active: self.security.is_active(),
            has_pin: self.security.is_pin_set(),
        }
    }
}

pub type SharedEngine = Arc<Mutex<DibaEngine>>;
