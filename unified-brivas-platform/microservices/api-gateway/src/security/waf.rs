//! Web Application Firewall (WAF)
//!
//! OWASP Top 10 protection.

use regex::Regex;

pub struct Waf {
    sql_injection_pattern: Regex,
    xss_pattern: Regex,
    path_traversal_pattern: Regex,
    enabled: bool,
}

impl Waf {
    pub fn new() -> Self {
        Self {
            sql_injection_pattern: Regex::new(r"(?i)(\b(SELECT|INSERT|UPDATE|DELETE|DROP|UNION|OR|AND)\b.*\b(FROM|INTO|WHERE|TABLE)\b)").unwrap(),
            xss_pattern: Regex::new(r"(?i)(<script|javascript:|on\w+\s*=)").unwrap(),
            path_traversal_pattern: Regex::new(r"(\.\./)").unwrap(),
            enabled: true,
        }
    }

    pub fn check(&self, input: &str) -> WafResult {
        if !self.enabled {
            return WafResult::Pass;
        }

        // Check SQL injection
        if self.sql_injection_pattern.is_match(input) {
            return WafResult::Block {
                rule: "SQL_INJECTION".to_string(),
                message: "Potential SQL injection detected".to_string(),
            };
        }

        // Check XSS
        if self.xss_pattern.is_match(input) {
            return WafResult::Block {
                rule: "XSS".to_string(),
                message: "Potential XSS attack detected".to_string(),
            };
        }

        // Check path traversal
        if self.path_traversal_pattern.is_match(input) {
            return WafResult::Block {
                rule: "PATH_TRAVERSAL".to_string(),
                message: "Path traversal attempt detected".to_string(),
            };
        }

        WafResult::Pass
    }

    pub fn check_request(&self, path: &str, body: Option<&str>, headers: &[(&str, &str)]) -> WafResult {
        // Check path
        if let WafResult::Block { .. } = self.check(path) {
            return self.check(path);
        }

        // Check body
        if let Some(body) = body {
            if let WafResult::Block { .. } = self.check(body) {
                return self.check(body);
            }
        }

        // Check headers
        for (_, value) in headers {
            if let WafResult::Block { .. } = self.check(value) {
                return self.check(value);
            }
        }

        WafResult::Pass
    }
}

impl Default for Waf {
    fn default() -> Self {
        Self::new()
    }
}

pub enum WafResult {
    Pass,
    Block { rule: String, message: String },
}
