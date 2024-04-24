//!
//! Test that a robot is alive by sending and receiving a packet from it
//! 

use embedded_hal::blocking::delay::DelayMs;
use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use robocup_base_station::{RADIO_CSN, RADIO_CE};

use rtic_nrf24l01::Radio;
use rtic_nrf24l01::config::power_amplifier::PowerAmplifier;

use robojackets_robocup_rtp::control_message::{ControlMessageBuilder, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::robot_status_message::{RobotStatusMessage, ROBOT_STATUS_SIZE};
use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::{BASE_STATION_ADDRESS, ROBOT_RADIO_ADDRESSES};

use packed_struct::{PackedStruct, PackedStructSlice};

// Power Amplifier Level
const PA_LEVEL: PowerAmplifier = PowerAmplifier::PALow;
// Channel to Send Packets on
const RF_CHANNEL: u8 = 0;

#[test]
/// Send Messages and wait for responses from the robot
fn test_robot_is_alive() {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let csn = gpio.get(RADIO_CSN).unwrap().into_output();
    let ce = gpio.get(RADIO_CE).unwrap().into_output();
    let mut delay = Delay::new();

    let mut radio = Radio::new(ce, csn);
    if let Err(err) = radio.begin(&mut spi, &mut delay) {
        panic!("Unable to Initialize the Radio: {:?}", err);
    }

    radio.set_pa_level(PA_LEVEL, &mut spi, &mut delay);
    radio.set_channel(RF_CHANNEL, &mut spi, &mut delay);
    radio.set_payload_size(CONTROL_MESSAGE_SIZE as u8, &mut spi, &mut delay);
    radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[0], &mut spi, &mut delay);
    radio.open_reading_pipe(1, BASE_STATION_ADDRESS, &mut spi, &mut delay);
    radio.start_listening(&mut spi, &mut delay);
    delay.delay_ms(1_000u32);
    radio.stop_listening(&mut spi, &mut delay);

    loop {
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

        for i in 0..5 {
            radio.stop_listening(&mut spi, &mut delay);
            let _ = radio.write(&packed_data, &mut spi, &mut delay);

            radio.start_listening(&mut spi, &mut delay);
            
            if radio.packet_ready(&mut spi, &mut delay) {
                let mut buffer = [0u8; ROBOT_STATUS_SIZE];
                radio.read_payload(&mut buffer, &mut spi, &mut delay);

                match RobotStatusMessage::unpack_from_slice(&buffer[..]) {
                    Ok(data) => println!("Received: {:?}", data),
                    Err(err) => println!("Unable to Unpack Data: {:?}", err),
                }

                radio.flush_rx(&mut spi, &mut delay);
                break;
            }

            delay.delay_ms(10u32);

            if i == 4 {
                println!("No Data Received This Iteration");
            }
        }

        delay.delay_ms(100u32);
    }
}