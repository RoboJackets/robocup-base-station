//!
//! Example program that prints the buffer coming from the xbox controller so I can
//! hopefully debug the buffer
//!

use std::{fs::File, io::Read, thread::sleep, time::Duration};

fn main() {
    loop {
        println!("Controller One");
        if let Ok(mut file) = File::open("/dev/input/js0") {
            let mut buffer = [0u8; 32];
            if file.read(&mut buffer).is_ok() {
                println!("{:?}", buffer);
            }
        }

        println!("Controller Two");
        if let Ok(mut file) = File::open("/dev/input/js1") {
            let mut buffer = [0u8; 32];
            if file.read(&mut buffer).is_ok() {
                println!("{:?}", buffer);
            }
        }

        sleep(Duration::from_millis(500));
    }
}
