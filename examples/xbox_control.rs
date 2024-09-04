//!
//! Example program that prints the buffer coming from the xbox controller so I can
//! hopefully debug the buffer
//!

use std::{
    fs::File,
    io::{Read, Write},
    thread::sleep,
    time::Duration,
};

const INPUT_DELAY_MS: u64 = 2_000;
const XPAD_PACKET_LENGTH: usize = 120;

/// Rust struct that is used to convert the inputs from controllers to usable inputs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct XboxControlCommand {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub start: bool,
    pub select: bool,
    pub xbox_button: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    pub left_trigger: bool,
    pub right_trigger: bool,
    pub lstick_x: i8,
    pub lstick_y: i8,
    pub rstick_x: i8,
    pub rstick_y: i8,
    pub dpad_up: bool,
    pub dpad_right: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
}

impl From<&[u8; XPAD_PACKET_LENGTH]> for XboxControlCommand {
    fn from(value: &[u8; XPAD_PACKET_LENGTH]) -> Self {
        Self {
            a: value[4] == 1,
            b: value[12] == 1,
            x: value[20] == 1,
            y: value[28] == 1,
            start: value[60] == 1,
            select: value[52] == 1,
            xbox_button: value[68] == 1,
            left_shoulder: value[36] == 1,
            right_shoulder: value[44] == 1,
            left_trigger: false,
            right_trigger: false,
            lstick_x: 0,
            lstick_y: 0,
            rstick_x: 0,
            rstick_y: 0,
            dpad_up: value[108] == 1,
            dpad_right: value[100] == 1,
            dpad_down: value[116] == 1,
            dpad_left: value[92] == 1,
        }
    }
}

fn main() {
    println!("Hold Left Stick Up");
    sleep(Duration::from_millis(1_000));
    let mut buffers = Vec::new();
    for i in 0..1_000 {
        if i % 100 == 0 {
            println!("{}", i);
        }
        if let Ok(mut file) = File::open("/dev/input/js0") {
            let mut buffer = [0u8; XPAD_PACKET_LENGTH];
            file.read(&mut buffer).unwrap();
            buffers.push(buffer);
        }

        sleep(Duration::from_millis(10));
    }

    println!("Writing to File");
    if let Ok(mut file) = File::create("./xbox/left_stick_up.txt") {
        for i in 0..1_000 {
            file.write(format!("{:?}\n", buffers[i]).as_bytes()).unwrap();
        }
    }

    loop {
        println!("Controller One");
        if let Ok(mut file) = File::open("/dev/input/js0") {
            let mut buffer = [0u8; XPAD_PACKET_LENGTH];
            if file.read(&mut buffer).is_ok() {
                let command = XboxControlCommand::from(&buffer);
                println!("{:?}", command);
            }
        }

        println!("Controller Two");
        if let Ok(mut file) = File::open("/dev/input/js1") {
            let mut buffer = [0u8; XPAD_PACKET_LENGTH];
            if file.read(&mut buffer).is_ok() {
                let command = XboxControlCommand::from(&buffer);
                println!("{:?}", command);
            }
        }

        sleep(Duration::from_millis(500));
    }
}
