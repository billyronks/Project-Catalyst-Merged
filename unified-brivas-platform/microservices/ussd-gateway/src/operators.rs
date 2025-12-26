//! Operator-specific rendering and configuration

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Nigerian telecom operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operator {
    Mtn,
    Airtel,
    Glo,
    NineMobile,
    Unknown,
}

impl Operator {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "mtn" => Self::Mtn,
            "airtel" => Self::Airtel,
            "glo" => Self::Glo,
            "9mobile" | "etisalat" => Self::NineMobile,
            _ => Self::Unknown,
        }
    }

    pub fn max_message_length(&self) -> usize {
        match self {
            Self::Mtn => 182,
            Self::Airtel => 160,
            Self::Glo => 160,
            Self::NineMobile => 160,
            Self::Unknown => 160,
        }
    }

    pub fn supports_back_navigation(&self) -> bool {
        match self {
            Self::Mtn => true,
            Self::Airtel => true,
            Self::Glo => false,
            Self::NineMobile => true,
            Self::Unknown => false,
        }
    }

    pub fn session_timeout_secs(&self) -> u64 {
        match self {
            Self::Mtn => 180,
            Self::Airtel => 120,
            Self::Glo => 180,
            Self::NineMobile => 120,
            Self::Unknown => 120,
        }
    }
}

/// Trait for operator-specific rendering
pub trait OperatorRenderer: Send + Sync {
    fn operator(&self) -> Operator;
    fn render_menu(&self, title: &str, options: &[(String, String)]) -> String;
    fn max_content_length(&self) -> usize;
    fn supports_unicode(&self) -> bool;
}

/// MTN Nigeria renderer
pub struct MtnNigeriaRenderer;

impl OperatorRenderer for MtnNigeriaRenderer {
    fn operator(&self) -> Operator {
        Operator::Mtn
    }

    fn render_menu(&self, title: &str, options: &[(String, String)]) -> String {
        let mut output = format!("{}\n", title);
        for (key, label) in options {
            output.push_str(&format!("{}. {}\n", key, label));
        }
        output
    }

    fn max_content_length(&self) -> usize {
        182
    }

    fn supports_unicode(&self) -> bool {
        true
    }
}

/// Airtel Nigeria renderer
pub struct AirtelNigeriaRenderer;

impl OperatorRenderer for AirtelNigeriaRenderer {
    fn operator(&self) -> Operator {
        Operator::Airtel
    }

    fn render_menu(&self, title: &str, options: &[(String, String)]) -> String {
        let mut output = format!("{}\n", title);
        for (key, label) in options {
            output.push_str(&format!("{}. {}\n", key, label));
        }
        output
    }

    fn max_content_length(&self) -> usize {
        160
    }

    fn supports_unicode(&self) -> bool {
        false
    }
}

/// Glo Nigeria renderer
pub struct GloNigeriaRenderer;

impl OperatorRenderer for GloNigeriaRenderer {
    fn operator(&self) -> Operator {
        Operator::Glo
    }

    fn render_menu(&self, title: &str, options: &[(String, String)]) -> String {
        let mut output = format!("{}\n", title);
        for (key, label) in options {
            output.push_str(&format!("{}) {}\n", key, label));
        }
        output
    }

    fn max_content_length(&self) -> usize {
        160
    }

    fn supports_unicode(&self) -> bool {
        false
    }
}

/// 9Mobile Nigeria renderer
pub struct NineMobileRenderer;

impl OperatorRenderer for NineMobileRenderer {
    fn operator(&self) -> Operator {
        Operator::NineMobile
    }

    fn render_menu(&self, title: &str, options: &[(String, String)]) -> String {
        let mut output = format!("{}\n", title);
        for (key, label) in options {
            output.push_str(&format!("{}. {}\n", key, label));
        }
        output
    }

    fn max_content_length(&self) -> usize {
        160
    }

    fn supports_unicode(&self) -> bool {
        true
    }
}

/// Get appropriate renderer for operator
pub fn get_renderer(operator: Operator) -> Box<dyn OperatorRenderer> {
    match operator {
        Operator::Mtn => Box::new(MtnNigeriaRenderer),
        Operator::Airtel => Box::new(AirtelNigeriaRenderer),
        Operator::Glo => Box::new(GloNigeriaRenderer),
        Operator::NineMobile => Box::new(NineMobileRenderer),
        Operator::Unknown => Box::new(MtnNigeriaRenderer), // Default to MTN
    }
}
