use std::{thread::sleep, time::Duration};

use ncomm::{node::Node, executor::{simple_multi_executor::SimpleMultiExecutor, Executor}};
use robocup_base_station::{cpu_relay_node::CpuRelayNode, robot_relay_node::RobotRelayNode};
use robojackets_robocup_rtp::Team;
use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::{Gpio, Trigger}, hal::Delay};
use sx127::{LoRa, RadioMode};

#[test]
fn test_round_trip() {
    let gpio = Gpio::new().unwrap();

    // Create Radio 1
    let spi0 = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let cs0 = gpio.get(8u8).unwrap().into_output();
    let reset0 = gpio.get(2u8).unwrap().into_output();
    let delay0 = Delay::new();

    let send_radio = LoRa::new(spi0, cs0, reset0, 915, delay0).unwrap();

    let mut cpu_relay_node = CpuRelayNode::new(
        "127.0.0.1:8000",
        send_radio,
        Team::Blue,
        1u8,
    );

    // Create Recieve Radio
    let spi1 = Spi::new(Bus::Spi1, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let cs1 = gpio.get(16u8).unwrap().into_output();
    let reset1 = gpio.get(26u8).unwrap().into_output();
    let delay1 = Delay::new();

    let mut receive_radio = LoRa::new(spi1, cs1, reset1, 915, delay1).unwrap();
    match receive_radio.set_mode(RadioMode::RxContinuous) {
        Ok(_) => println!("Listening"),
        Err(_) => panic!("Unable to set to listening"),
    }

    let mut robot_relay_node = RobotRelayNode::new(
        "127.0.0.1:8001",
        vec!["10.42.0.1:8000"],
        receive_radio,
        Team::Blue,
        1u8,
    );

    // Enable Receive Interrupt
    let mut radio_interrupt = gpio.get(13u8).unwrap().into_input();
    radio_interrupt.set_async_interrupt(Trigger::RisingEdge, move |_| {
        println!("Received Data");
        robot_relay_node.update();
    }).unwrap();

    let mut executor = SimpleMultiExecutor::new_with(
        vec![
            ("Cpu Relay Thread", &mut cpu_relay_node),
        ]
    );

    executor.start();
    executor.update_loop();
}