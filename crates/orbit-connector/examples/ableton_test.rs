use orbit_connector::ableton::osc_handler::OscHandler;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Orbit Ableton OSC Test");
    println!("======================");
    println!("Make sure:");
    println!("1. Ableton Live is running");
    println!("2. OrbitRemote is selected as Control Surface in Preferences");
    println!();

    let osc = OscHandler::new(12000, "127.0.0.1", 11000)?;

    println!("Starting playback...");
    osc.play()?;
    thread::sleep(Duration::from_secs(2));

    println!("Setting tempo to 120 BPM...");
    osc.set_tempo(150.0)?;
    thread::sleep(Duration::from_secs(1));

    println!("Setting track 0 volume to 0.5...");
    osc.set_track_volume(0, 0.5)?;
    thread::sleep(Duration::from_secs(1));

    println!("Muting track 0...");
    osc.set_track_mute(0, true)?;
    thread::sleep(Duration::from_secs(2));

    println!("Unmuting track 0...");
    osc.set_track_mute(0, false)?;
    thread::sleep(Duration::from_secs(1));

    println!("Stopping playback...");
    osc.stop()?;

    println!("\nTest complete!");
    println!("Check Ableton's Log.txt file for detailed OSC messages");

    Ok(())
}
