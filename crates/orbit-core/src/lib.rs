use serde::{Deserialize, Serialize};

/// Shared data structures and constants between Orbit crates

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitMessage {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    MidiGeneration,
    StemSeparation,
    ScreenContext,
    AudioContext,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiGenerationRequest {
    pub prompt: String,
    pub style: Option<String>,
    pub duration_bars: Option<u32>,
    pub bpm: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemSeparationRequest {
    pub audio_path: String,
    pub output_path: String,
    pub stems: Vec<StemType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StemType {
    Drums,
    Bass,
    Vocals,
    Other,
}

impl OrbitMessage {
    pub fn new(message_type: MessageType, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            message_type,
            payload,
        }
    }
}
