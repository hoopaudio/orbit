pub mod osc_handler;
pub mod live_api;

use std::net::SocketAddr;
use orbit_core::{OrbitMessage, AbletonCommand};
use crate::ableton::osc_handler::OscHandler;

pub struct AbletonConnector {
    #[allow(dead_code)]
    osc_address: SocketAddr,
    is_connected: bool,
    osc_handler: Option<OscHandler>,
}

impl AbletonConnector {
    pub fn new(host: &str, port: u16) -> Self {
        let addr = format!("{}:{}", host, port)
            .parse()
            .expect("Failed to parse OSC address");

        Self {
            osc_address: addr,
            is_connected: false,
            osc_handler: None,
        }
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let osc_handler = OscHandler::new(12000, "127.0.0.1", 11000)
            .map_err(|e| anyhow::anyhow!("Failed to create OSC handler: {}", e))?;
        self.osc_handler = Some(osc_handler);
        self.is_connected = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        self.osc_handler = None;
        self.is_connected = false;
    }

    pub async fn send_message(&self, _msg: OrbitMessage) -> anyhow::Result<()> {
        if !self.is_connected {
            return Err(anyhow::anyhow!("Not connected to Ableton Live"));
        }

        Ok(())
    }

    pub async fn execute_command(&self, command: AbletonCommand) -> anyhow::Result<Option<serde_json::Value>> {
        if !self.is_connected {
            return Err(anyhow::anyhow!("Not connected to Ableton Live"));
        }

        let osc = self.osc_handler.as_ref().ok_or_else(|| anyhow::anyhow!("OSC handler not initialized"))?;

        match command {
            AbletonCommand::Play => {
                osc.play().map_err(|e| anyhow::anyhow!("Play command failed: {}", e))?;
                Ok(Some(serde_json::json!({"status": "playing"})))
            }
            AbletonCommand::Stop => {
                osc.stop().map_err(|e| anyhow::anyhow!("Stop command failed: {}", e))?;
                Ok(Some(serde_json::json!({"status": "stopped"})))
            }
            AbletonCommand::SetTempo { bpm } => {
                osc.set_tempo(bpm).map_err(|e| anyhow::anyhow!("Set tempo failed: {}", e))?;
                Ok(Some(serde_json::json!({"tempo": bpm})))
            }
            AbletonCommand::SetTrackVolume { track_id, volume } => {
                osc.set_track_volume(track_id as i32, volume).map_err(|e| anyhow::anyhow!("Set track volume failed: {}", e))?;
                Ok(Some(serde_json::json!({"track_id": track_id, "volume": volume})))
            }
            AbletonCommand::SetTrackMute { track_id, mute } => {
                osc.set_track_mute(track_id as i32, mute).map_err(|e| anyhow::anyhow!("Set track mute failed: {}", e))?;
                Ok(Some(serde_json::json!({"track_id": track_id, "mute": mute})))
            }
            _ => {
                Err(anyhow::anyhow!("Command {:?} not yet implemented", command))
            }
        }
    }
}