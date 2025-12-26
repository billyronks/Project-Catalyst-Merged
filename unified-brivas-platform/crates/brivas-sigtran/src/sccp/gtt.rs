//! Global Title Translation

use super::address::{GlobalTitle, SccpAddress};
use crate::errors::SccpError;
use std::collections::HashMap;
use std::sync::RwLock;

/// Global Title Translator
pub struct GlobalTitleTranslator {
    /// Translation rules: GT prefix -> Point Code
    rules: RwLock<HashMap<String, u32>>,
    /// Default point code for unknown translations
    default_pc: Option<u32>,
}

impl GlobalTitleTranslator {
    /// Create new GTT
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            default_pc: None,
        }
    }

    /// Add translation rule
    pub fn add_rule(&self, prefix: &str, point_code: u32) {
        let mut rules = self.rules.write().unwrap();
        rules.insert(prefix.to_string(), point_code);
    }

    /// Set default point code
    pub fn set_default(&mut self, pc: u32) {
        self.default_pc = Some(pc);
    }

    /// Translate SCCP address to destination point code
    pub fn translate(&self, address: &SccpAddress) -> Result<u32, SccpError> {
        // If point code is already present, use it
        if let Some(pc) = address.point_code {
            return Ok(pc);
        }

        // Otherwise, translate based on Global Title
        if let Some(ref gt) = address.global_title {
            let digits = gt.digits();
            let rules = self.rules.read().unwrap();
            
            // Find longest matching prefix
            let mut best_match: Option<(usize, u32)> = None;
            
            for (prefix, pc) in rules.iter() {
                if digits.starts_with(prefix) {
                    if best_match.is_none() || prefix.len() > best_match.unwrap().0 {
                        best_match = Some((prefix.len(), *pc));
                    }
                }
            }

            if let Some((_, pc)) = best_match {
                return Ok(pc);
            }
        }

        // Try default
        self.default_pc.ok_or(SccpError::NoTranslation)
    }

    /// Load rules from configuration
    pub fn load_rules(&self, rules: &[(String, u32)]) {
        let mut r = self.rules.write().unwrap();
        for (prefix, pc) in rules {
            r.insert(prefix.clone(), *pc);
        }
    }
}

impl Default for GlobalTitleTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtt_translation() {
        let gtt = GlobalTitleTranslator::new();
        gtt.add_rule("234", 1001); // Nigeria
        gtt.add_rule("2348", 1002); // Nigeria mobile

        let addr = SccpAddress::from_gt(
            GlobalTitle::e164("2348012345678"),
            Some(6),
        );

        let pc = gtt.translate(&addr).unwrap();
        assert_eq!(pc, 1002); // Should match longer prefix

        let addr2 = SccpAddress::from_gt(
            GlobalTitle::e164("2340123456789"),
            Some(6),
        );
        let pc2 = gtt.translate(&addr2).unwrap();
        assert_eq!(pc2, 1001); // Falls back to shorter prefix
    }
}
