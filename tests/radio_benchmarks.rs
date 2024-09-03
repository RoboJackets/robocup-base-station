//!
//! Various Benchmarks for the Radio
//! 

use embedded_hal::blocking::delay::DelayMs;
use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use robocup_base_station::{RADIO_CSN, RADIO_CE};

use rtic_nrf24l01::Radio;
use rtic_nrf24l01::config::*;

use robojackets_robocup_rtp::control_message::{ControlMessageBuilder, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::robot_status_message::{RobotStatusMessage, ROBOT_STATUS_SIZE};
use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::{BASE_STATION_ADDRESS, ROBOT_RADIO_ADDRESSES};

use packed_struct::{PackedStruct, PackedStructSlice};

use rand::random;

// Power Amplifier Level
const PA_LEVEL: power_amplifier::PowerAmplifier = power_amplifier::PowerAmplifier::PALow;
// Total Number of Messages to Send
const TOTAL_MESSAGES: usize = 100;
// Delay Between Subsequent Packets
const PACKET_DELAY_MS: u32 = 50;
// Channel to Send Packets on
const RF_CHANNEL: u8 = 106;

#[test]
/// Send TOTAL_MESSAGES packets at a delay of PACKET_DELAY_MS with a PA of
/// PA_LEVEL over the channel RF_CHANNEL recording the number of packets that
/// are acknowledged by the receiver.
fn benchmark_radio_send() {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let csn = gpio.get(RADIO_CSN).unwrap().into_output();
    let ce = gpio.get(RADIO_CE).unwrap().into_output();
    let mut delay = Delay::new();

    let mut radio = Radio::new(ce, csn);
    if radio.begin(&mut spi, &mut delay).is_err() {
        panic!("Unable to Initialize the Radio");
    }

    radio.set_pa_level(PA_LEVEL, &mut spi, &mut delay);
    radio.set_channel(RF_CHANNEL, &mut spi, &mut delay);
    radio.set_payload_size(CONTROL_MESSAGE_SIZE as u8, &mut spi, &mut delay);
    radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[5], &mut spi, &mut delay);
    radio.open_reading_pipe(1, BASE_STATION_ADDRESS, &mut spi, &mut delay);
    radio.stop_listening(&mut spi, &mut delay);

    let mut acknowledged_packets = 0;
    for _ in 0..TOTAL_MESSAGES {
        let control_message = ControlMessageBuilder::new()
            .team(Team::Blue)
            .robot_id(0)
            .body_x(random())
            .body_y(random())
            .body_w(random())
            .build();

        let packed_data = match control_message.pack() {
            Ok(bytes) => bytes,
            Err(err) => panic!("Unable to Pack Data: {:?}", err),
        };

        let acknowledged = radio.write(&packed_data, &mut spi, &mut delay);

        radio.flush_tx(&mut spi, &mut delay);
        if acknowledged {
            acknowledged_packets += 1;
        }

        delay.delay_ms(PACKET_DELAY_MS);
    }

    println!("{} / {} Packets Were Acknowledged", acknowledged_packets, TOTAL_MESSAGES);
}

#[test]
/// Receive Constantly Checking for new Data every PACKET_DELAY_MS
fn benchmark_radio_receive() {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)
        .expect("Unable to Acquire SPI Peripherals");
    let gpio = Gpio::new().expect("Unable to Acquire GPIO Peripherals");
    let csn = gpio.get(RADIO_CSN).expect("Unable to get Radio CNS").into_output();
    let ce = gpio.get(RADIO_CE).expect("Unable to Get Radio CE").into_output();
    let mut delay = Delay::new();

    let mut radio = Radio::new(ce, csn);
    radio.begin(&mut spi, &mut delay).expect("Unable to Initialize the radio");
    radio.set_pa_level(PA_LEVEL, &mut spi, &mut delay);
    radio.set_channel(RF_CHANNEL, &mut spi, &mut delay);
    radio.set_payload_size(ROBOT_STATUS_SIZE as u8, &mut spi, &mut delay);
    radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[0], &mut spi, &mut delay);
    radio.open_reading_pipe(1, BASE_STATION_ADDRESS, &mut spi, &mut delay);
    radio.start_listening(&mut spi, &mut delay);

    loop {
        if radio.packet_ready(&mut spi, &mut delay) {
            let mut buffer = [0u8; ROBOT_STATUS_SIZE];
            radio.read(&mut buffer, &mut spi, &mut delay);

            match RobotStatusMessage::unpack_from_slice(&buffer[..]) {
                Ok(data) => println!("Received: {:?}", data),
                Err(err) => println!("Unable to Unpack Data: {:?}", err),
            }

            radio.flush_rx(&mut spi, &mut delay);
        }

        delay.delay_ms(PACKET_DELAY_MS);
    }
}
