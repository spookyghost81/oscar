use std::collections::HashMap;

use anyhow::Result;
use rosc::{OscMessage, OscPacket};
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;

pub enum TrackControl {
    Volume(f32),
    Pan(f32),
    ToggleMute,
    ToggleSolo,
    ToggleRecordArm,
}

pub struct FxState {
    pub name: String,
    pub preset: String,
}

pub struct TrackState {
    pub volume: f32,
    pub vu_lr: f32,
    pub vu_l: f32,
    pub vu_r: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub record_armed: bool,
}

pub struct PlayState {
    pub is_playing: bool,
    pub position_seconds: f32,
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
    let mut received_packets = HashMap::new();
    let start_time = std::time::Instant::now();
    loop {
        let mut buf = [0u8; 1024];
        let (len, addr) = socket.recv_from(&mut buf).await?;
        //println!("Received {} bytes from {}: {:?}", len, addr, &buf[..len]);

        let packet = rosc::decoder::decode_udp(&buf)?;

        match packet.1 {
            OscPacket::Message(msg) => {
                if !received_packets.contains_key(&msg.addr) {
                    received_packets.insert(msg.addr.clone(), Vec::new());
                }
                println!("Received OSC message: {:?}", msg.addr);
                let args = received_packets.get_mut(&msg.addr).unwrap();
                for arg in msg.args {
                    println!("  Arg: {:?}", arg);
                    args.push(arg);
                }
            }
            OscPacket::Bundle(bundle) => {
                //println!("Received OSC bundle: {:?}", bundle);
                for packet in bundle.content {
                    match packet {
                        OscPacket::Message(msg) => {
                            println!("  Bundle message: {:?}", msg.addr);
                            if !received_packets.contains_key(&msg.addr) {
                                received_packets.insert(msg.addr.clone(), Vec::new());
                            }
                            let args = received_packets.get_mut(&msg.addr).unwrap();
                            for arg in msg.args {
                                println!("    Arg: {:?}", arg);
                                args.push(arg);
                            }
                        }
                        OscPacket::Bundle(bundle) => {
                            println!("  Nested bundle: {:?}", bundle);
                        }
                    }
                }
            }
        }

        let current_time = std::time::Instant::now();
        if current_time.duration_since(start_time) > std::time::Duration::from_secs(10) {
            break;
        }
    }

    let mut output = std::fs::File::create("output.txt")?;
    for (addr, args) in received_packets {
        println!("{}", addr);
        writeln!(output, "{}", addr)?;
        for arg in args {
            println!("  Arg: {:?}", arg);
            writeln!(output, "  Arg: {:?}", arg)?;
        }
    }
    output.flush()?;

    Ok(())
}
