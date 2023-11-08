use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::{LoRa, RadioMode};

#[test]
fn test_receive() {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let cs = gpio.get(0u8).unwrap().into_output();
    let reset = gpio.get(1u8).unwrap().into_output();
    let delay = Delay::new();

    // Create Radio
    let mut radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();

    match radio.set_tx_power(17, 1) {
        Ok(_) => println!("Successfully set TX power"),
        Err(_) => panic!("Error Setting Tx Power"),
    }

    match radio.set_mode(RadioMode::RxContinuous) {
        Ok(_) => println!("Listening"),
        Err(_) => panic!("Couldn't set radio to receive"),
    }

    loop {
        match radio.read_packet() {
            Ok(buffer) => {
                for c in buffer {
                    if c == 0x0 {
                        break;
                    }
                    println!("got: {}", c as char);
                }
            },
            Err(_) => panic!("Error while reading data"),
        }
    }
}