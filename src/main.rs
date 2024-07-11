// SPDX-License-Identifier: WTFPL
// Copyright 2024 rtldg <rtldg@protonmail.com>

use byteorder::{LittleEndian, WriteBytesExt};
use std::{io::Write, time::Duration};

#[repr(C)]
#[derive(Debug, Default)]
struct PlayerInfoWeCareAbout {
    origin: [f32; 3],
    angles: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Default)]
struct Frame {
    pos: [f32; 3],
    ang: [f32; 2],
    buttons: u32,
    flags: u32,
    movetype: u32,
}

// cargo run -- "[U:1:123]" "bhop_tranquility.replay" 55.466 350 6300

// search in cheat engine for the tick from demoui until you find this...
const PLAYBACK_TICK_OFFSET: usize = 0x45b724;
// search in cheat engine for values from `cl_showpos 1` until you find this
const EYEPOS_AND_ANG_OFFSET: usize = 0x4FCA80;
// search in cheat engine for m_fFlags.
// stand on the ground, search for 257, +duck in console, search for 263, loop
// TODO: this doesn't work right... idk... have to actually find entities in memory correctly...
//const FLAGS_ADDRESS: usize = 0x0B4E67D4;

fn main() {
    let hl2 = proc_mem::Process::with_name("hl2.exe").unwrap();
    let client_dll = hl2.module("client.dll").unwrap();
    let engine_dll = hl2.module("engine.dll").unwrap();

    let mut frames = vec![];
    let mut last_playback_tick = 0;

    let mut args = std::env::args().skip(1);
    let steamid = args.next().unwrap();
    let replayfilename = args.next().unwrap();
    let runtime = args.next().unwrap();
    let runtime = runtime.parse::<f32>().unwrap();
    let start_tick = args.next().unwrap().parse::<u32>().unwrap();
    let end_tick = args.next().unwrap().parse::<u32>().unwrap();

    println!("waiting for tick {start_tick}...");

    loop {
        let playback_tick = hl2
            .read_mem::<u32>(engine_dll.base_address() + PLAYBACK_TICK_OFFSET)
            .unwrap();

        if playback_tick > end_tick {
            break;
        }

        if playback_tick < start_tick || playback_tick == last_playback_tick {
            std::thread::sleep(Duration::from_millis(1));
        } else {
            last_playback_tick = playback_tick;

            let mut info = hl2
                .read_mem::<PlayerInfoWeCareAbout>(client_dll.base_address() + EYEPOS_AND_ANG_OFFSET)
                .unwrap();
            // this origin is actually eye-pos it seems...
            info.origin[2] -= 64.0;

            // let flags = hl2.read_mem::<u32>(FLAGS_ADDRESS).unwrap();
            let flags = 0;
            let is_ducking = flags & 2 != 0;
            const IN_JUMP: u32 = 1 << 1;
            const IN_DUCK: u32 = 1 << 2;
            const FL_ONGROUND: u32 = 1 << 0;
            const FL_DUCKING: u32 = 1 << 1;
            const FL_CLIENT: u32 = 1 << 7; // sourcemod flag
            const MOVETYPE_WALK: u32 = 2;
            const MOVETYPE_NOCLIP: u32 = 8;

            frames.push(Frame {
                pos: info.origin,
                ang: [info.angles[0], info.angles[1]],
                buttons: IN_JUMP | if is_ducking { IN_DUCK } else { 0 },
                flags: flags & (FL_DUCKING | FL_ONGROUND) | FL_CLIENT,
                movetype: if flags & FL_ONGROUND != 0 {
                    MOVETYPE_WALK
                } else {
                    MOVETYPE_NOCLIP
                },
            });

            if frames.len() == 1 {
                println!("started!");
            }
            // println!("{:?} {is_ducking}", info);
        }
    }

    let mut replay = std::fs::File::create(&replayfilename).unwrap();
    replay
        .write_all(b"2:{SHAVITREPLAYFORMAT}{FINAL}\n")
        .unwrap();
    replay
        .write_u32::<LittleEndian>(frames.len() as u32)
        .unwrap();
    replay.write_f32::<LittleEndian>(runtime).unwrap();
    replay
        .write_all(format!("{}\0", steamid).as_bytes())
        .unwrap();
    replay
        .write_all(unsafe {
            std::slice::from_raw_parts(
                frames.as_ptr() as *const u8,
                frames.len() * std::mem::size_of::<Frame>(),
            )
        })
        .unwrap();

    println!("wrote to {}", replayfilename);
}
