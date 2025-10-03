use rosc::{OscPacket, OscMessage, OscType};
use std::net::UdpSocket;
use std::time::Duration;

pub struct OscHandler {
    socket: UdpSocket,
    ableton_addr: String,
}

impl OscHandler {
    pub fn new(local_port: u16, ableton_host: &str, ableton_port: u16) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", local_port))?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;

        Ok(Self {
            socket,
            ableton_addr: format!("{}:{}", ableton_host, ableton_port),
        })
    }

    pub fn send(&self, address: &str, args: Vec<OscType>) -> Result<(), Box<dyn std::error::Error>> {
        let msg = OscMessage {
            addr: address.to_string(),
            args,
        };

        let packet = OscPacket::Message(msg);
        let buf = rosc::encoder::encode(&packet)?;

        self.socket.send_to(&buf, &self.ableton_addr)?;
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<OscMessage>, Box<dyn std::error::Error>> {
        let mut buf = [0u8; rosc::decoder::MTU];

        match self.socket.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size])?;

                if let OscPacket::Message(msg) = packet {
                    Ok(Some(msg))
                } else {
                    Ok(None)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn play(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send("/live/play", vec![])
    }

    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send("/live/stop", vec![])
    }

    pub fn set_track_volume(&self, track_id: i32, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.send("/live/track/volume", vec![
            OscType::Int(track_id),
            OscType::Float(volume),
        ])
    }

    pub fn set_track_mute(&self, track_id: i32, mute: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.send("/live/track/mute", vec![
            OscType::Int(track_id),
            OscType::Int(if mute { 1 } else { 0 }),
        ])
    }

    pub fn set_tempo(&self, bpm: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.send("/live/tempo", vec![OscType::Float(bpm)])
    }
}