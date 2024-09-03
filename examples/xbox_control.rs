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

fn main() {
    println!("Left Stick Up");
    sleep(Duration::from_millis(1_000));
    let mut left_stick_up_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut left_stick_up_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Left Stick Down");
    sleep(Duration::from_millis(1_000));
    let mut left_stick_down_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut left_stick_down_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Left Stick Right");
    sleep(Duration::from_millis(1_000));
    let mut left_stick_right_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut left_stick_right_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Left Stick Left");
    sleep(Duration::from_millis(1_000));
    let mut left_stick_left_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut left_stick_left_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    if let Ok(mut file) = File::create("./xbox/left_stick.txt") {
        file.write(format!("Up: {:?}", left_stick_up_buffer).as_bytes())
            .unwrap();
        file.write(format!("Right: {:?}", left_stick_right_buffer).as_bytes())
            .unwrap();
        file.write(format!("Down: {:?}", left_stick_down_buffer).as_bytes())
            .unwrap();
        file.write(format!("Left: {:?}", left_stick_left_buffer).as_bytes())
            .unwrap();
    }

    println!("X Button");
    sleep(Duration::from_millis(1_000));
    let mut x_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut x_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Y Button");
    sleep(Duration::from_millis(1_000));
    let mut y_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut y_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("A Button");
    sleep(Duration::from_millis(1_000));
    let mut a_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut a_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("B Button");
    sleep(Duration::from_millis(1_000));
    let mut b_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut b_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    if let Ok(mut file) = File::create("./xbox/buttons.txt") {
        file.write(format!("A: {:?}", a_buffer).as_bytes()).unwrap();
        file.write(format!("B: {:?}", b_buffer).as_bytes()).unwrap();
        file.write(format!("X: {:?}", x_buffer).as_bytes()).unwrap();
        file.write(format!("Y: {:?}", y_buffer).as_bytes()).unwrap();
    }

    println!("Left Bumper");
    sleep(Duration::from_millis(1_000));
    let mut left_bumper_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut left_bumper_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Left Trigger");
    sleep(Duration::from_millis(1_000));
    let mut left_trigger_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut left_trigger_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Right Bumper");
    sleep(Duration::from_millis(1_000));
    let mut right_bumper_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut right_bumper_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    println!("Right Trigger");
    sleep(Duration::from_millis(1_000));
    let mut right_trigger_buffer = [0u8; 32];
    match File::open("/dev/input/js0") {
        Ok(mut file) => {
            if let Err(err) = file.read(&mut right_trigger_buffer) {
                eprintln!("Error reading from /dev/input/js0: {:?}", err);
            }
        }
        Err(err) => eprintln!("Error opening /dev/input/js0: {:?}", err),
    }

    if let Ok(mut file) = File::create("./xbox/triggers.txt") {
        file.write(format!("LB: {:?}", left_bumper_buffer).as_bytes())
            .unwrap();
        file.write(format!("LT: {:?}", left_trigger_buffer).as_bytes())
            .unwrap();
        file.write(format!("RB: {:?}", right_bumper_buffer).as_bytes())
            .unwrap();
        file.write(format!("RT: {:?}", right_trigger_buffer).as_bytes())
            .unwrap();
    }

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
