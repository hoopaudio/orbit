use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i32,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub armed: bool,
    pub color: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: i32,
    pub name: String,
    pub track_id: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub length: f64,
    pub is_playing: bool,
    pub is_recording: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub track_id: i32,
    pub is_active: bool,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub id: i32,
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub is_quantized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: i32,
    pub name: String,
    pub tempo: f32,
    pub is_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSet {
    pub name: String,
    pub tempo: f32,
    pub time_signature: (i32, i32),
    pub is_playing: bool,
    pub tracks: Vec<Track>,
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveCommand {
    Play,
    Stop,
    Record,
    SetTempo(f32),
    SetTrackVolume {
        track_id: i32,
        volume: f32,
    },
    SetTrackPan {
        track_id: i32,
        pan: f32,
    },
    SetTrackMute {
        track_id: i32,
        mute: bool,
    },
    SetTrackSolo {
        track_id: i32,
        solo: bool,
    },
    ArmTrack {
        track_id: i32,
        armed: bool,
    },
    LaunchClip {
        track_id: i32,
        clip_id: i32,
    },
    StopClip {
        track_id: i32,
        clip_id: i32,
    },
    LaunchScene {
        scene_id: i32,
    },
    SetDeviceParameter {
        device_id: i32,
        param_id: i32,
        value: f32,
    },
    GetLiveSet,
    GetTrack {
        track_id: i32,
    },
    GetClip {
        track_id: i32,
        clip_id: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveResponse {
    Success,
    Error(String),
    LiveSet(LiveSet),
    Track(Track),
    Clip(Clip),
    Device(Device),
}

impl LiveCommand {
    pub fn to_osc_path(&self) -> String {
        match self {
            LiveCommand::Play => "/live/play".to_string(),
            LiveCommand::Stop => "/live/stop".to_string(),
            LiveCommand::Record => "/live/record".to_string(),
            LiveCommand::SetTempo(_) => "/live/tempo".to_string(),
            LiveCommand::SetTrackVolume { .. } => "/live/track/volume".to_string(),
            LiveCommand::SetTrackPan { .. } => "/live/track/pan".to_string(),
            LiveCommand::SetTrackMute { .. } => "/live/track/mute".to_string(),
            LiveCommand::SetTrackSolo { .. } => "/live/track/solo".to_string(),
            LiveCommand::ArmTrack { .. } => "/live/track/arm".to_string(),
            LiveCommand::LaunchClip { .. } => "/live/clip/launch".to_string(),
            LiveCommand::StopClip { .. } => "/live/clip/stop".to_string(),
            LiveCommand::LaunchScene { .. } => "/live/scene/launch".to_string(),
            LiveCommand::SetDeviceParameter { .. } => "/live/device/param".to_string(),
            LiveCommand::GetLiveSet => "/live/get".to_string(),
            LiveCommand::GetTrack { .. } => "/live/track/get".to_string(),
            LiveCommand::GetClip { .. } => "/live/clip/get".to_string(),
        }
    }
}
