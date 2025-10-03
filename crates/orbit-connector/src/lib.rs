// CLAP plugin for DAW integration
// TODO: Implement full CLAP plugin when clap-rs adds CLAP audio plugin support
// For now, this is a placeholder structure

pub mod ableton;

use anyhow::Result;
use orbit_core::*;
use ableton::AbletonConnector;

/// Orbit Connector - CLAP plugin for DAW integration
pub struct OrbitConnector {
    pub osc_host: String,
    pub osc_port: u16,
    ableton: Option<AbletonConnector>,
}

impl OrbitConnector {
    pub fn new() -> Self {
        Self {
            osc_host: "127.0.0.1".to_string(),
            osc_port: 8000,
            ableton: None,
        }
    }

    pub async fn connect_ableton(&mut self) -> Result<()> {
        let mut ableton = AbletonConnector::new("127.0.0.1", 11000);
        ableton.connect().await?;
        self.ableton = Some(ableton);
        log::info!("Connected to Ableton Live");
        Ok(())
    }

    pub async fn disconnect_ableton(&mut self) {
        if let Some(mut ableton) = self.ableton.take() {
            ableton.disconnect().await;
            log::info!("Disconnected from Ableton Live");
        }
    }

    pub async fn send_midi_generation_request(&self, prompt: &str) -> Result<()> {
        log::info!("Sending MIDI generation request: {}", prompt);
        // TODO: Implement OSC communication with Orbit app
        Ok(())
    }

    pub async fn send_stem_separation_request(&self, audio_path: &str) -> Result<()> {
        log::info!("Sending stem separation request for: {}", audio_path);
        // TODO: Implement OSC communication with Orbit app
        Ok(())
    }

    pub async fn handle_message(&self, message: OrbitMessage) -> Result<Option<OrbitMessage>> {
        match message.message_type {
            MessageType::AbletonControl => {
                if let Some(ableton) = &self.ableton {
                    self.handle_ableton_control(ableton, message).await
                } else {
                    Ok(Some(OrbitMessage::new(
                        MessageType::Status,
                        serde_json::json!({
                            "error": "Ableton not connected"
                        }),
                    )))
                }
            }
            _ => {
                log::info!("Unhandled message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }

    async fn handle_ableton_control(
        &self,
        ableton: &AbletonConnector,
        message: OrbitMessage,
    ) -> Result<Option<OrbitMessage>> {
        let request: AbletonControlRequest = serde_json::from_value(message.payload)?;

        let response = match self.execute_ableton_command(ableton, request.command).await {
            Ok(data) => AbletonControlResponse {
                success: true,
                message: None,
                data,
            },
            Err(e) => AbletonControlResponse {
                success: false,
                message: Some(e.to_string()),
                data: None,
            },
        };

        Ok(Some(OrbitMessage::new(
            MessageType::Status,
            serde_json::to_value(response)?,
        )))
    }

    async fn execute_ableton_command(
        &self,
        ableton: &AbletonConnector,
        command: AbletonCommand,
    ) -> Result<Option<serde_json::Value>> {
        ableton.execute_command(command).await
    }
}

impl Default for OrbitConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::{AbletonCommand, MessageType};

    #[tokio::test]
    async fn test_orbit_connector_creation() {
        let connector = OrbitConnector::new();
        assert_eq!(connector.osc_host, "127.0.0.1");
        assert_eq!(connector.osc_port, 8000);
        assert!(connector.ableton.is_none());
    }

    #[tokio::test]
    async fn test_ableton_control_message_creation() {
        let play_msg = OrbitMessage::ableton_control(AbletonCommand::Play);
        assert_eq!(play_msg.message_type, MessageType::AbletonControl);

        let request: AbletonControlRequest = serde_json::from_value(play_msg.payload).unwrap();
        matches!(request.command, AbletonCommand::Play);
    }

    #[tokio::test]
    async fn test_ableton_control_message_with_params() {
        let tempo_msg = OrbitMessage::ableton_control(AbletonCommand::SetTempo { bpm: 120.0 });
        assert_eq!(tempo_msg.message_type, MessageType::AbletonControl);

        let request: AbletonControlRequest = serde_json::from_value(tempo_msg.payload).unwrap();
        if let AbletonCommand::SetTempo { bpm } = request.command {
            assert_eq!(bpm, 120.0);
        } else {
            panic!("Expected SetTempo command");
        }
    }

    #[tokio::test]
    async fn test_handle_message_without_ableton_connection() {
        let connector = OrbitConnector::new();
        let play_msg = OrbitMessage::ableton_control(AbletonCommand::Play);

        let response = connector.handle_message(play_msg).await.unwrap().unwrap();
        assert_eq!(response.message_type, MessageType::Status);

        let error_json = response.payload.as_object().unwrap();
        assert_eq!(error_json["error"], "Ableton not connected");
    }

    #[tokio::test]
    async fn test_handle_non_ableton_message() {
        let connector = OrbitConnector::new();
        let status_msg = OrbitMessage::new(MessageType::Status, serde_json::json!({}));

        let response = connector.handle_message(status_msg).await.unwrap();
        assert!(response.is_none());
    }

    #[test]
    fn test_ableton_command_serialization() {
        let commands = vec![
            AbletonCommand::Play,
            AbletonCommand::Stop,
            AbletonCommand::SetTempo { bpm: 140.0 },
            AbletonCommand::SetTrackVolume { track_id: 1, volume: 0.8 },
            AbletonCommand::SetTrackMute { track_id: 2, mute: true },
        ];

        for command in commands {
            let serialized = serde_json::to_value(&command).unwrap();
            let deserialized: AbletonCommand = serde_json::from_value(serialized).unwrap();
            // Test that serialization round-trip works
            assert_eq!(
                serde_json::to_string(&command).unwrap(),
                serde_json::to_string(&deserialized).unwrap()
            );
        }
    }
}