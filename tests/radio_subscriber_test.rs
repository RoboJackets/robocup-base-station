use std::{sync::{Arc, Mutex}, thread::sleep, time::Duration};

use ncomm::publisher_subscriber::Receive;
use robocup_base_station::robot_relay_node::radio_subscriber::RadioSubscriber;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::{LoRa, RadioMode};

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
    let mut radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();
    match radio.set_mode(RadioMode::RxContinuous) {
        Ok(_) => println!("Listening"),
        Err(_) => panic!("Couldn't set radio to receive"),
    }
    // Wrap Radio in Mutex
    let radio = Arc::new(Mutex::new(radio));

    let mut radio_subscriber: RadioSubscriber<Spi, rppal::gpio::OutputPin, rppal::gpio::OutputPin, Delay, rppal::spi::Error, RobotStatusMessage> = RadioSubscriber::new(radio);

    let mut radio_interrupt = gpio.get(2u8).unwrap().into_input();
    radio_interrupt.set_async_interrupt(rppal::gpio::Trigger::RisingEdge, move |_| {
        radio_subscriber.update_data();

        for e in radio_subscriber.data.drain(..) {
            println!("Found Data: {:?}", e);
        }
    }).unwrap();

    loop {
        sleep(Duration::from_secs(1));
    }
}