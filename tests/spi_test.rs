use std::{thread::sleep, time::Duration};

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::LoRa;

use embedded_hal::blocking::spi::{Transfer, Write};

#[test]
fn test_base_radio_send_hello() {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let cs = gpio.get(8u8).unwrap().into_output();
    let reset = gpio.get(21u8).unwrap().into_output();
    let delay = Delay::new();

    // Create Radio
    let mut radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();

    match radio.set_tx_power(17, 1) {
        Ok(_) => println!("Successfully set TX power"),
        Err(_) => panic!("Error Setting Tx Power"),
    }

    let message = "IT WORKS!!";
    let mut buffer = [0;255];
    for (i, c) in message.chars().enumerate() {
        buffer[i] = c as u8;
    }

    for _ in 0..10 {
        println!("Sending Message: {}", message);

        match radio.transmit_payload_busy(buffer, message.len()) {
            Ok(packet_size) => println!("Sent packet with size: {}", packet_size),
            Err(_) => panic!("Error sending packet"),
        }

        sleep(Duration::from_secs(1));
    }
}
