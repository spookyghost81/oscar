use std::collections::HashMap;

use anyhow::Result;
use rosc::{OscMessage, OscPacket};
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub enum TrackControl {
    Volume(f32),
    Pan(f32),
    ToggleMute,
    ToggleSolo,
    ToggleRecordArm,
    Custom(String, Vec<rosc::OscType>),
}

impl mlua::UserData for TrackControl {}

#[derive(Debug, Default)]
pub struct FxState {
    pub name: String,
    pub preset: String,
    pub bypassed: bool,
}

#[derive(Debug, Default)]
pub struct TrackState {
    pub name: String,
    pub volume: f32,
    pub vu_lr: f32,
    pub vu_l: f32,
    pub vu_r: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub record_armed: bool,
    pub fx_bypassed: bool,
    pub fx: Vec<FxState>,
}

#[derive(Debug, Default)]
pub struct PlayState {
    pub is_playing: bool,
    pub beats_str: String,
    pub time_str: String,
    pub time_seconds: f32,
}

#[derive(Debug, Default)]
pub struct DawState {
    pub play_state: PlayState,
    pub master_track: TrackState,
    pub tracks: [TrackState; 8],
}

pub fn handle_packet(packet: OscPacket, daw_state: &mut DawState) {
    match packet {
        OscPacket::Message(msg) => {
            //println!("Received OSC message: {:?}", msg.addr);
            let parts = msg
                .addr
                .split('/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            let track_target = match parts.as_slice() {
                ["master", rest @ ..] => Some((&mut daw_state.master_track, rest)),
                ["track", track_num_str, rest @ ..] => {
                    if let Ok(track_num) = track_num_str.parse::<usize>() {
                        if track_num > 0 && track_num <= daw_state.tracks.len() {
                            Some((&mut daw_state.tracks[track_num - 1], rest))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                ["beat", "str"] => {
                    if let Some(rosc::OscType::String(beat_str)) = msg.args.get(0) {
                        //println!("Current beat: {}", beat_str);
                        daw_state.play_state.beats_str = beat_str.clone();
                    }
                    return;
                }
                ["time", "str"] => {
                    if let Some(rosc::OscType::String(time_str)) = msg.args.get(0) {
                        //println!("Current time: {}", time_str);
                        daw_state.play_state.time_str = time_str.clone();
                    }
                    return;
                }
                ["time"] => {
                    if let Some(rosc::OscType::Float(time_sec)) = msg.args.get(0) {
                        //println!("Current time: {} seconds", time_sec);
                        daw_state.play_state.time_seconds = *time_sec;
                    }
                    return;
                }
                ["play"] => {
                    daw_state.play_state.is_playing = true;
                    return;
                }
                ["stop"] => {
                    daw_state.play_state.is_playing = false;
                    return;
                }
                // ["position", "seconds"] => {
                //     if let Some(rosc::OscType::Float(pos)) = msg.args.get(0) {
                //         daw_state.play_state.position_seconds = *pos;
                //         println!("Current position: {} seconds", pos);
                //     }
                //     return;
                // }
                ["lastmarker", "name"] => {
                    if let Some(rosc::OscType::String(marker_name)) = msg.args.get(0) {
                        println!("Last marker: {}", marker_name);
                    }
                    return;
                }
                _ => None,
            };

            if track_target.is_none() {
                println!("Unhandled Track message: {:?}", msg.addr);
                for arg in msg.args {
                    println!("  Arg: {:?}", arg);
                }
                return;
            }

            let (track, rest) = track_target.unwrap();

            match rest {
                ["name"] => {
                    if let Some(rosc::OscType::String(name)) = msg.args.get(0) {
                        track.name = name.clone();
                        println!("Track name: {}", track.name);
                    }
                }
                ["volume", "db"] => {
                    if let Some(rosc::OscType::Float(vol)) = msg.args.get(0) {
                        track.volume = *vol;
                        println!("Track {} volume: {} dB", track.name, track.volume);
                    }
                }
                ["vu"] => {
                    if let Some(rosc::OscType::Float(vu_lr)) = msg.args.get(0) {
                        track.vu_lr = *vu_lr;
                        println!("Track {} VU LR: {}", track.name, track.vu_lr);
                    }
                }
                ["vu", "L"] => {
                    if let Some(rosc::OscType::Float(vu_l)) = msg.args.get(0) {
                        track.vu_l = *vu_l;
                        println!("Track {} VU L: {}", track.name, track.vu_l);
                    }
                }
                ["vu", "R"] => {
                    if let Some(rosc::OscType::Float(vu_r)) = msg.args.get(0) {
                        track.vu_r = *vu_r;
                        println!("Track {} VU R: {}", track.name, track.vu_r);
                    }
                }
                ["pan"] => {
                    if let Some(rosc::OscType::Float(pan)) = msg.args.get(0) {
                        track.pan = *pan;
                        println!("Track {} pan: {}", track.name, track.pan);
                    }
                }
                ["mute"] => {
                    if let Some(rosc::OscType::Float(mute)) = msg.args.get(0) {
                        track.mute = *mute != 0.0;
                        println!("Track {} mute: {}", track.name, track.mute);
                    }
                }
                ["solo"] => {
                    if let Some(rosc::OscType::Float(solo)) = msg.args.get(0) {
                        track.solo = *solo != 0.0;
                        println!("Track {} solo: {}", track.name, track.solo);
                    }
                }
                _ => {
                    //println!("Unhandled OSC message: {:?}", msg.addr);
                    // for arg in msg.args {
                    //     println!("  Arg: {:?}", arg);
                    // }
                }
            }
        }
        OscPacket::Bundle(bundle) => {
            //println!("Received OSC bundle: {:?}", bundle);
            for packet in bundle.content {
                handle_packet(packet, daw_state);
            }
        }
    }
}

#[tokio::test]
async fn test_daw_control() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect("127.0.0.1:8656").await?;
    // let mut stream = TcpStream::connect("localhost:8656").await?;
    let msg = OscMessage {
        addr: "/device/track/count".to_string(),
        args: vec![rosc::OscType::Int(8)],
    };

    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet)?;
    socket.send(&buf).await?;

    let msg = OscMessage {
        addr: "/device/track/bank/select".to_string(),
        args: vec![rosc::OscType::Int(1)],
    };

    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet)?;
    socket.send(&buf).await?;

    let mut daw_state = DawState::default();
    daw_state.master_track.name = "Master".to_string();

    for i in 1..=8 {
        let msg = OscMessage {
            addr: "/device/track/bank/select".to_string(),
            args: vec![rosc::OscType::Int(i)],
        };

        let packet = OscPacket::Message(msg);
        let buf = rosc::encoder::encode(&packet)?;
        socket.send(&buf).await?;
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    //let mut received_packets = HashMap::new();
    let start_time = std::time::Instant::now();
    loop {
        let mut buf = [0u8; 1024];
        let (len, addr) = socket.recv_from(&mut buf).await?;
        //println!("Received {} bytes from {}: {:?}", len, addr, &buf[..len]);

        let packet = rosc::decoder::decode_udp(&buf)?;
        handle_packet(packet.1, &mut daw_state);

        // let current_time = std::time::Instant::now();
        // if current_time.duration_since(start_time) > std::time::Duration::from_secs(10) {
        //     break;
        // }
    }

    // let mut output = std::fs::File::create("output.txt")?;
    // for (addr, args) in received_packets {
    //     println!("{}", addr);
    //     writeln!(output, "{}", addr)?;
    //     for arg in args {
    //         println!("  Arg: {:?}", arg);
    //         writeln!(output, "  Arg: {:?}", arg)?;
    //     }
    // }
    // output.flush()?;

    Ok(())
}
