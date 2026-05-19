use anyhow::Result;
use convert_case::{Case, Casing};
use rosc::{OscMessage, OscPacket};
use std::collections::HashMap;
use std::io::Write;
use std::net::UdpSocket;
use strum::{IntoEnumIterator, VariantIterator, VariantMetadata};

// use tokio::io::AsyncWriteExt;
// use tokio::net::TcpStream;
// use tokio::net::UdpSocket;

#[derive(
    Debug, Clone, strum_macros::AsRefStr, strum_macros::EnumIter, strum_macros::VariantNames,
)]
pub enum TrackControl {
    None,
    Volume(f32),
    Pan(f32),
    Mute(bool),
    Solo(bool),
    RecordArm(bool),
    Custom(String, Vec<rosc::OscType>),
}

impl Default for TrackControl {
    fn default() -> Self {
        TrackControl::None
    }
}

#[derive(
    Debug, Clone, strum_macros::AsRefStr, strum_macros::EnumIter, strum_macros::VariantNames,
)]
pub enum PlaybackControl {
    None,
    Play,
    Stop,
    Pause,
    Record,
    Rewind,
    FastForward,
    JumpToPreviousMarker,
    JumpToNextMarker,
    JumpToStart,
    JumpToEnd,
    JumpForwardBars(u32),
    JumpBackwardBars(u32),
}

impl Default for PlaybackControl {
    fn default() -> Self {
        PlaybackControl::None
    }
}

#[derive(
    Debug, Clone, strum_macros::AsRefStr, strum_macros::EnumIter, strum_macros::VariantNames,
)]
pub enum DawControlMessage {
    None,
    TrackControl(usize, TrackControl), // track number (1-based), control
    PlaybackControl(PlaybackControl),
    Custom(String, Vec<rosc::OscType>),
}

impl Default for DawControlMessage {
    fn default() -> Self {
        DawControlMessage::None
    }
}

impl DawControlMessage {
    pub fn track_control(track_num: usize, control_type: &str, value: f32) -> Result<Self> {
        let mut variant = TrackControl::iter()
            .find(|variant| variant.as_ref().to_case(Case::Snake) == control_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid track control type: {}", control_type))?;
        match variant {
            TrackControl::Volume(_) => variant = TrackControl::Volume(value),
            TrackControl::Pan(_) => variant = TrackControl::Pan(value),
            TrackControl::Mute(_) => variant = TrackControl::Mute(value != 0.0),
            TrackControl::Solo(_) => variant = TrackControl::Solo(value != 0.0),
            TrackControl::RecordArm(_) => variant = TrackControl::RecordArm(value != 0.0),
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported track control type: {}",
                    control_type
                ));
            }
        }
        Ok(DawControlMessage::TrackControl(track_num, variant))
    }

    pub fn playback_control(control_type: &str) -> Result<Self> {
        let variant = PlaybackControl::iter()
            .find(|variant| variant.as_ref().to_case(Case::Snake) == control_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid playback control type: {}", control_type))?;
        Ok(DawControlMessage::PlaybackControl(variant))
    }
}

#[test]
fn print_variants() {
    TrackControl::iter().for_each(|variant| {
        let conv = variant.as_ref().to_case(Case::Snake);
        println!("Variant: {} -> {}", variant.as_ref(), conv);
    });
}

#[test]
fn print_playback_variants() {
    PlaybackControl::iter().for_each(|variant| {
        let conv = variant.as_ref().to_case(Case::Snake);
        println!("Variant: {} -> {}", variant.as_ref(), conv);
    });
}

impl DawControlMessage {
    pub fn to_osc(&self) -> OscMessage {
        let mut addr = String::new();
        let mut args = Vec::new();
        match self {
            Self::TrackControl(track_num, control) => {
                addr = format!("/track/{}/", track_num);
                match control {
                    TrackControl::Volume(vol) => {
                        addr.push_str("volume/db");
                        args.push(rosc::OscType::Float(*vol));
                    }
                    TrackControl::Pan(pan) => {
                        addr.push_str("pan");
                        args.push(rosc::OscType::Float(*pan));
                    }
                    TrackControl::Mute(mute) => {
                        addr.push_str("mute");
                        args.push(rosc::OscType::Int(*mute as i32));
                    }
                    TrackControl::Solo(solo) => {
                        addr.push_str("solo");
                        args.push(rosc::OscType::Int(*solo as i32));
                    }
                    TrackControl::RecordArm(record_arm) => {
                        addr.push_str("record_arm");
                        args.push(rosc::OscType::Int(*record_arm as i32));
                    }
                    _ => {}
                }
            }
            Self::PlaybackControl(control) => match control {
                PlaybackControl::Play => addr = "/play".to_string(),
                PlaybackControl::Stop => addr = "/stop".to_string(),
                _ => {}
            },
            Self::Custom(custom_addr, custom_args) => {
                addr = custom_addr.clone();
                args.extend_from_slice(custom_args);
            }
            Self::None => {
                log::warn!("Attempted to convert DawControlMessage::None to OSC message");
            }
        }
        OscMessage { addr, args }
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct FxState {
    pub name: String,
    pub preset: String,
    pub bypassed: bool,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct PlayState {
    pub is_playing: bool,
    pub beats_str: String,
    pub time_str: String,
    pub time_seconds: f32,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct DawState {
    pub play_state: PlayState,
    pub master_track: TrackState,
    pub tracks: [TrackState; 8],
}

impl mlua::UserData for DawState {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "send_track_message",
            |lua, this, (track_index, track_control_type, value): (usize, String, mlua::Value)| {
                if track_index == 0 || track_index > this.tracks.len() {
                    return Err(mlua::Error::RuntimeError(format!(
                        "Invalid track index: {}. Must be between 1 and {}.",
                        track_index,
                        this.tracks.len()
                    )));
                }

                return Ok(());
            },
        );
    }
}

pub fn network_thread(
    sender: std::sync::mpsc::Sender<OscMessage>,
    receiver: std::sync::mpsc::Receiver<DawControlMessage>,
) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("127.0.0.1:8656")?;

    let msg = OscMessage {
        addr: "/device/track/count".to_string(),
        args: vec![rosc::OscType::Int(8)],
    };

    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet)?;
    socket.send(&buf)?;

    let msg = OscMessage {
        addr: "/device/track/bank/select".to_string(),
        args: vec![rosc::OscType::Int(1)],
    };

    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet)?;
    socket.send(&buf)?;

    let mut daw_state = DawState::default();
    daw_state.master_track.name = "Master".to_string();

    for i in 1..=8 {
        let msg = OscMessage {
            addr: "/device/track/bank/select".to_string(),
            args: vec![rosc::OscType::Int(i)],
        };

        let packet = OscPacket::Message(msg);
        let buf = rosc::encoder::encode(&packet)?;
        socket.send(&buf)?;
    }

    let msg = OscMessage {
        addr: "/device/track/bank/select".to_string(),
        args: vec![rosc::OscType::Int(1)],
    };

    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet)?;
    socket.send(&buf)?;

    std::thread::sleep(std::time::Duration::from_secs(1));
    //let mut received_packets = HashMap::new();
    let start_time = std::time::Instant::now();
    loop {
        let mut buf = [0u8; 1024];
        let (len, addr) = socket.recv_from(&mut buf)?;
        //println!("Received {} bytes from {}: {:?}", len, addr, &buf[..len]);

        let packet = rosc::decoder::decode_udp(&buf[..len])?;
        let leftover_bytes = packet.0.len();
        if leftover_bytes > 1 {
            println!(
                "Warning: {} bytes of data were not parsed in the OSC packet",
                leftover_bytes
            );
        }
        handle_packet(packet.1, sender.clone());

        receiver.try_iter().for_each(|msg| {
            let osc_msg = msg.to_osc();
            let packet = OscPacket::Message(osc_msg);
            let buf = rosc::encoder::encode(&packet).unwrap();
            socket.send(&buf).unwrap();
        });
    }

    Ok(())
}

fn handle_packet(packet: OscPacket, tx: std::sync::mpsc::Sender<OscMessage>) {
    match packet {
        OscPacket::Message(msg) => {
            //println!("Received OSC message: {:?}", msg.addr);
            tx.send(msg).unwrap();
        }
        OscPacket::Bundle(bundle) => {
            //println!("Received OSC bundle: {:?}", bundle);
            for packet in bundle.content {
                handle_packet(packet, tx.clone());
            }
        }
    }
}

impl DawState {
    pub fn update_from_osc_message(&mut self, msg: OscMessage) {
        update_daw_state(msg, self);
    }
}

fn update_daw_state(msg: OscMessage, daw_state: &mut DawState) {
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
        ["recarm"] => {
            if let Some(rosc::OscType::Float(recarm)) = msg.args.get(0) {
                track.record_armed = *recarm != 0.0;
                println!("Track {} record armed: {}", track.name, track.record_armed);
            }
        }
        _ => {
            println!("Unhandled OSC message: {:?}", msg.addr);
            for arg in msg.args {
                println!("  Arg: {:?}", arg);
            }
        }
    }
}
