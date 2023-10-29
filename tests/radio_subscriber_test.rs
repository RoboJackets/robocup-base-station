use std::{sync::{Arc, Mutex}, thread::sleep, time::Duration};

use robocup_base_station::robot_relay_node::radio_subscriber::RadioSubscriber;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::LoRa;

use robojackets_robocup_rtp::robot_status_message::RobotStatusMessage;

#[test]
fn send_data() {
    // Get Peripherals
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let cs = gpio.get(0u8).unwrap().into_output();
    let reset = gpio.get(1u8).unwrap().into_output();
    let delay = Delay::new();

    // Create Radio
    let radio = LoRa::new(spi, cs, reset, 1_000_000, delay).unwrap();

    // Wrap Radio in Mutex
    let radio = Arc::new(Mutex::new(radio));

    let mut radio_subscriber = RadioSubscriber::new(radio);

    while radio_subscriber.data.is_empty() {
        sleep(Duration::from_secs(1));
    }

    for e in radio_subscriber.data.drain(..).into_iter().enumerate() {
        let (_, data): (_, RobotStatusMessage) = e;
        println!("Received: {:?}", data);
    }
}