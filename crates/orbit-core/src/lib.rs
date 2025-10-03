use serde::{Deserialize, Serialize};

/// Shared data structures and constants between Orbit crates

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitMessage {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    MidiGeneration,
    StemSeparation,
    ScreenContext,
    AudioContext,
    Status,
    AbletonControl,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbletonControlRequest {
    pub command: AbletonCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbletonCommand {
    // Transport Controls
    Play,
    Stop,
    Record,
    SetTempo { bpm: f32 },

    // Track Controls
    SetTrackVolume { track_id: u32, volume: f32 },
    SetTrackPan { track_id: u32, pan: f32 },
    SetTrackMute { track_id: u32, mute: bool },
    SetTrackSolo { track_id: u32, solo: bool },
    ArmTrack { track_id: u32, armed: bool },

    // Clip Controls
    LaunchClip { track_id: u32, clip_id: u32 },
    StopClip { track_id: u32, clip_id: u32 },

    // Scene Controls
    LaunchScene { scene_id: u32 },

    // Device Controls
    SetDeviceParameter { device_id: u32, param_id: u32, value: f32 },

    // Info Queries
    GetSessionInfo,
    GetTrack { track_id: u32 },
    GetClip { track_id: u32, clip_id: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbletonControlResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
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

    pub fn ableton_control(command: AbletonCommand) -> Self {
        let request = AbletonControlRequest { command };
        Self::new(
            MessageType::AbletonControl,
            serde_json::to_value(request).unwrap_or_default(),
        )
    }
}
