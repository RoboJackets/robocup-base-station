use std::{thread::sleep, time::Duration};

use ncomm::publisher_subscriber::Receive;
use robocup_base_station::robot_relay_node::radio_subscriber::RadioSubscriber;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::{LoRa, RadioMode};

use robojackets_robocup_rtp::robot_status_message::RobotStatusMessage;

#[test]
fn send_data() {
    // Get Peripherals
    let spi = Spi::new(Bus::Spi1, SlaveSelect::Ss2, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let cs = gpio.get(16u8).unwrap().into_output();
    let reset = gpio.get(26u8).unwrap().into_output();
    let delay = Delay::new();

    // Create Radio
    let mut radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();
    match radio.set_mode(RadioMode::RxContinuous) {
        Ok(_) => println!("Listening"),
        Err(_) => panic!("Couldn't set radio to receive"),
    }

    let mut radio_subscriber: RadioSubscriber<Spi, rppal::gpio::OutputPin, rppal::gpio::OutputPin, Delay, rppal::spi::Error, RobotStatusMessage> = RadioSubscriber::new(radio);

    let mut radio_interrupt = gpio.get(13u8).unwrap().into_input();
    radio_interrupt.set_async_interrupt(rppal::gpio::Trigger::RisingEdge, move |_| {
        println!("Interrupt");

        radio_subscriber.update_data();

        for e in radio_subscriber.data.drain(..) {
            println!("Found Data: {:?}", e);
        }
    }).unwrap();

    loop {
        sleep(Duration::from_secs(1));
    }
}