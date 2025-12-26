//! Conference Layout Management
//!
//! Manages video conference layouts (grid, speaker focus, etc.)

use brivas_video_sdk::ConferenceLayout;
use uuid::Uuid;

/// Layout configuration
pub struct LayoutConfig {
    pub layout: ConferenceLayout,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub background_color: String,
    pub border_color: String,
    pub border_size: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layout: ConferenceLayout::Auto,
            canvas_width: 1920,
            canvas_height: 1080,
            background_color: "#1a1a2e".to_string(),
            border_color: "#4a4a6a".to_string(),
            border_size: 2,
        }
    }
}

/// Layout manager for conference video composition
pub struct LayoutManager {
    /// FreeSWITCH ESL connection
    freeswitch_esl: String,
}

impl LayoutManager {
    pub fn new(freeswitch_esl: &str) -> Self {
        Self {
            freeswitch_esl: freeswitch_esl.to_string(),
        }
    }

    /// Get layout positions for participants
    pub fn get_positions(&self, layout: ConferenceLayout, participant_count: usize) -> Vec<LayoutPosition> {
        match layout {
            ConferenceLayout::Grid => self.calculate_grid_positions(participant_count),
            ConferenceLayout::SpeakerFocus => self.calculate_speaker_focus_positions(participant_count),
            ConferenceLayout::Presentation => self.calculate_presentation_positions(participant_count),
            ConferenceLayout::Gallery => self.calculate_gallery_positions(participant_count),
            ConferenceLayout::Sidebar => self.calculate_sidebar_positions(participant_count),
            ConferenceLayout::Auto => {
                // Auto-select based on participant count
                if participant_count <= 4 {
                    self.calculate_grid_positions(participant_count)
                } else {
                    self.calculate_speaker_focus_positions(participant_count)
                }
            }
        }
    }

    /// Apply layout to conference via FreeSWITCH
    pub async fn apply_layout(
        &self,
        conference_id: Uuid,
        layout: ConferenceLayout,
    ) -> Result<(), LayoutError> {
        // TODO: Send ESL command to FreeSWITCH
        tracing::info!(
            conference_id = %conference_id,
            layout = ?layout,
            "Applying layout"
        );
        Ok(())
    }

    fn calculate_grid_positions(&self, count: usize) -> Vec<LayoutPosition> {
        let columns = (count as f64).sqrt().ceil() as usize;
        let rows = (count + columns - 1) / columns;
        let tile_width = 1920.0 / columns as f64;
        let tile_height = 1080.0 / rows as f64;

        (0..count)
            .map(|i| {
                let row = i / columns;
                let col = i % columns;
                LayoutPosition {
                    x: (col as f64 * tile_width) as u32,
                    y: (row as f64 * tile_height) as u32,
                    width: tile_width as u32,
                    height: tile_height as u32,
                    z_index: 0,
                }
            })
            .collect()
    }

    fn calculate_speaker_focus_positions(&self, count: usize) -> Vec<LayoutPosition> {
        let mut positions = Vec::with_capacity(count);
        
        // Main speaker takes 75% of width
        positions.push(LayoutPosition {
            x: 0,
            y: 0,
            width: 1440,
            height: 810,
            z_index: 0,
        });

        // Others in sidebar
        let sidebar_count = count.saturating_sub(1);
        if sidebar_count > 0 {
            let tile_height = 810 / sidebar_count.min(5) as u32;
            for i in 0..sidebar_count.min(5) {
                positions.push(LayoutPosition {
                    x: 1440,
                    y: i as u32 * tile_height,
                    width: 480,
                    height: tile_height,
                    z_index: 1,
                });
            }
        }

        positions
    }

    fn calculate_presentation_positions(&self, count: usize) -> Vec<LayoutPosition> {
        self.calculate_speaker_focus_positions(count)
    }

    fn calculate_gallery_positions(&self, count: usize) -> Vec<LayoutPosition> {
        self.calculate_grid_positions(count)
    }

    fn calculate_sidebar_positions(&self, count: usize) -> Vec<LayoutPosition> {
        self.calculate_speaker_focus_positions(count)
    }
}

#[derive(Debug, Clone)]
pub struct LayoutPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub z_index: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Conference not found")]
    ConferenceNotFound,
    #[error("Layout not supported")]
    NotSupported,
    #[error("ESL error: {0}")]
    EslError(String),
}
