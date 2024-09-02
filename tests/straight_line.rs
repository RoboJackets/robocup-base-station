//!
//! Move the robots in a straight line
//! 

use embedded_hal::blocking::delay::DelayMs;
use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use robocup_base_station::{RADIO_CSN, RADIO_CE, BASE_AMPLIFICATION_LEVEL, CHANNEL};

use rtic_nrf24l01::Radio;

use robojackets_robocup_rtp::control_message::{ControlMessageBuilder, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::{BASE_STATION_ADDRESS, ROBOT_RADIO_ADDRESSES};

use packed_struct::PackedStruct;

#[test]
fn straight_line_test() {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let csn = gpio.get(RADIO_CSN).unwrap().into_output();
    let ce = gpio.get(RADIO_CE).unwrap().into_output();
    let mut delay = Delay::new();

    let mut radio = Radio::new(ce, csn);
    if radio.begin(&mut spi, &mut delay).is_err() {
        panic!("Unable to Initialize the Radio");
    }

    radio.set_pa_level(BASE_AMPLIFICATION_LEVEL, &mut spi, &mut delay);
    radio.set_channel(CHANNEL, &mut spi, &mut delay);
    radio.set_payload_size(CONTROL_MESSAGE_SIZE as u8, &mut spi, &mut delay);
    radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[0], &mut spi, &mut delay);
    radio.open_reading_pipe(1, BASE_STATION_ADDRESS, &mut spi, &mut delay);
    radio.stop_listening(&mut spi, &mut delay);

    // Move forward for 2 seconds
    for _ in 0..40 {
        println!("Forward");
        let control_message = ControlMessageBuilder::new()
            .team(Team::Blue)
            .robot_id(0)
            .body_x(0.0)
            .body_y(1.0)
            .body_w(0.0)
            .build();
        
        let packed_data = match control_message.pack() {
            Ok(bytes) => bytes,
            Err(err) => panic!("Unable to Pack Data: {:?}", err),
        };

        let ack = radio.write(&packed_data, &mut spi, &mut delay);
        if !ack {
            println!("No Ack");
        }

        radio.flush_tx(&mut spi, &mut delay);
        
        delay.delay_ms(50u32);
    }

    let control_message = ControlMessageBuilder::new()
        .team(Team::Blue)
        .robot_id(0)
        .body_x(0.0)
        .body_y(0.0)
        .body_w(0.0)
        .build();

    let packed_data = match control_message.pack() {
        Ok(bytes) => bytes,
        Err(err) => panic!("Unable to Pack Data: {:?}", err),
    };

    let _ = radio.write(&packed_data, &mut spi, &mut delay);
    radio.flush_tx(&mut spi, &mut delay);
    println!("Done");
}