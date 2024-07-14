use std::{thread, time::Duration};

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use rtic_nrf24l01::Radio;
use rtic_nrf24l01::config::*;

#[test]
fn ping_pong() {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let csn = gpio.get(8).unwrap().into_output();
    let ce = gpio.get(22).unwrap().into_output();
    let mut delay = Delay::new();

    let mut radio = Radio::new(ce, csn);

    if let Err(err) = radio.begin(&mut spi, &mut delay) {
        println!("Configuration: {:?}", radio.get_registers(&mut spi, &mut delay));
        thread::sleep(Duration::from_millis(500));
        println!("Configuration: {:?}", radio.get_registers(&mut spi, &mut delay));
        panic!("Unable to initialize the radio: {:?}", err);
    }

    radio.set_pa_level(power_amplifier::PowerAmplifier::PALow, &mut spi, &mut delay);
    radio.set_channel(106, &mut spi, &mut delay);

    radio.set_payload_size(4, &mut spi, &mut delay);

    radio.open_writing_pipe([0xC3, 0xC3, 0xC3, 0xC3, 0xC1], &mut spi, &mut delay);

    radio.open_reading_pipe(1, [0xE7, 0xE7, 0xE7, 0xE7, 0xE7], &mut spi, &mut delay);

    radio.start_listening(&mut spi, &mut delay);

    thread::sleep(Duration::from_millis(1_000));

    println!("Configuration: {:?}", radio.get_registers(&mut spi, &mut delay));
    
    radio.stop_listening(&mut spi, &mut delay);

    let mut listening = false;
    let mut payload = 0;
    loop {
        if listening {
            if radio.available(&mut spi, &mut delay) {
                let mut read_buffer = [0u8; 4];
                radio.read(&mut read_buffer, &mut spi, &mut delay);
                payload = from_bytes(&read_buffer);
                println!("Received: {}", payload);

                listening = false;
                radio.stop_listening(&mut spi, &mut delay);
                payload += 1;
            }
        } else {
            let p = to_bytes(&payload);
            let report = radio.write(&p, &mut spi, &mut delay);

            if report {
                println!("Sent: {}", payload);
                listening = true;
                radio.start_listening(&mut spi, &mut delay);
            } else {
                println!("Transmission Timed Out");
            }
        }

        thread::sleep(Duration::from_millis(1_000));
    }
}

fn to_bytes(value: &u32) -> [u8; 4] {
    [
        (value & 0xFF) as u8,
        ((value & (0xFF << 8)) >> 8) as u8,
        ((value & (0xFF << 16)) >> 16) as u8,
        ((value & (0xFF << 24)) >> 24) as u8,
    ]
}

fn from_bytes(value: &[u8]) -> u32 {
    (value[0] as u32) | ((value[1] as u32) << 8) |
        ((value[2] as u32) << 16) | ((value[3] as u32) << 24)
}