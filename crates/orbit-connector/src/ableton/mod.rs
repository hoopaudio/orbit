pub mod osc_handler;
pub mod live_api;

use std::net::SocketAddr;
use orbit_core::OrbitMessage;

pub struct AbletonConnector {
    #[allow(dead_code)]
    osc_address: SocketAddr,
    is_connected: bool,
}

impl AbletonConnector {
    pub fn new(host: &str, port: u16) -> Self {
        let addr = format!("{}:{}", host, port)
            .parse()
            .expect("Failed to parse OSC address");

        Self {
            osc_address: addr,
            is_connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_connected = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        self.is_connected = false;
    }

    pub async fn send_message(&self, _msg: OrbitMessage) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_connected {
            return Err("Not connected to Ableton Live".into());
        }

        Ok(())
    }
}