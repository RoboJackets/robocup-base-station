//!
//! The basic principle of the base station is to have two NComm nodes that do the following:
//! 
//! 1. Receive commands from the Field Computer and forward said commands to the robots
//! 2. Receive information from the Robots and forward alive robot information to the base computer
//! 
//! The Send Radio uses SPI0 and the following pinout:
//! -> SCLK = GPIO 11 (pin 23)
//! -> MISO = GPIO 9 (pin 21)
//! -> MOSI = GPIO 10 (pin 19)
//! -> CSN = GPIO 8 (pin 24)
//! -> RESET = GPIO 2 (pin 3)
//! 
//! The Receive Radio Uses SPI1 and the following pinout:
//! -> SCLK = GPIO 21 (pin 40)
//! -> MISO = GPIO 19 (pin 35)
//! -> MOSI = GPIO 20 (pin 38)
//! -> CSN = GPIO 16 (pin 36)
//! -> RESET = GPIO 26 (pin 37)
//! -> IRQ = GPIO 13 (pin 33)
//! 
//! THe sx127x radio only has 1 Fifo so the volume of transmissions on it will cause
//! the send or receive data to be overwritten which is why there are two here.
//! 

use std::error::Error;
use std::sync::{Arc, Mutex};

use ncomm::node::Node;
use ncomm::executor::{Executor, simple_multi_executor::SimpleMultiExecutor};

use robocup_base_station::cpu_relay_node::CpuRelayNode;
use robocup_base_station::robot_relay_node::RobotRelayNode;
use robocup_base_station::timeout_checker::TimeoutCheckerNode;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use robojackets_robocup_rtp::Team;

use sx127::{LoRa, RadioMode};

use clap::Parser;

/// The Arguments passed to the base station program.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The address of the base computer udp socket to publish robot status messages to
    #[arg(required = true)]
    pub base_computer_address: String,

    // The address on the raspberry pi computer to bind the udp socket that is 
    // listening to incoming data from the base computer
    #[arg(default_value_t = String::from("0.0.0.0:8000"))]
    pub receive_bind_address: String,
    
    // The address on the raspberry pi computer to bind the udp socket that is
    // sending data to the base computer
    #[arg(default_value_t = String::from("0.0.0.0:8001"))]
    pub send_bind_address: String,

    // is the blue team
    // TODO (Nathaniel Wert): Combine Blue and Yellow to be a singular flag that parses the team
    // color
    #[arg(short, long, default_value_t = false)]
    pub blue: bool,

    // Boolean value to determine whether the team this base station is sending commands to
    // is the yellow team
    // TODO (Nathaniel Wert): Combine Blue and Yellow to be a singular flag that parses the team
    // color
    #[arg(short, long, default_value_t = false)]
    pub yellow: bool,

    // The number of robots in play (most likely either 6 or 11)
    #[arg(short, long, default_value_t = 6)]
    pub robots: u8,

    // The length in milliseconds of the timeout before we consider a robot dead
    #[arg(short, long, default_value_t = 5_000)]
    pub timeout: u128,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Parse the team
    let team = match (args.blue, args.yellow) {
        (true, _) | (false, false) => Team::Blue,
        (false, true) => Team::Yellow,
    };

    // Get Peripherals
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
    let gpio = Gpio::new()?;
    let cs = gpio.get(8u8)?.into_output();
    let reset = gpio.get(2u8)?.into_output();
    let delay = Delay::new();

    // Create Radio
    let radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();
    let radio = Arc::new(Mutex::new(radio));

    // Create Receive Radio
    let spi = Spi::new(Bus::Spi1, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
    let cs = gpio.get(16u8)?.into_output();
    let reset = gpio.get(26u8)?.into_output();
    let delay = Delay::new();

    let mut receive_radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();
    match receive_radio.set_mode(RadioMode::RxContinuous) {
        Ok(_) => println!("Listening"),
        Err(_) => panic!("Unable to set to listening"),
    }

    // Create the process that receives commands from the base computer and relays such commands to the robots
    let mut cpu_relay_node = CpuRelayNode::new(
        args.receive_bind_address.as_str(),
        radio.clone(),
        team,
        args.robots,
    );

    // Create the process that receives status messages from the robots and relays that information to the base computer
    // This node has a static lifetime because it is going to be controlled by an interrupt.
    let send_bind_address = Box::new(args.send_bind_address);
    let base_computer_address = Box::new(args.base_computer_address);
    let mut robot_relay_node = RobotRelayNode::new(
        Box::leak(send_bind_address),
        vec![
            Box::leak(base_computer_address),
        ],
        receive_radio,
        team,
        args.robots,
    );

    let subscribers = robot_relay_node.create_subscriber();
    println!("Subscribers: {}", subscribers.len());

    // Create the process that keeps up to date with reviving and sleeping the robots
    let mut timeout_checker = TimeoutCheckerNode::new(
        radio.clone(),
        subscribers,
        team,
        args.robots,
        args.timeout,
    );

    // Enable Interrupt on GPIO 2 for receiving and transmitting information from the robots
    let mut radio_interrupt = gpio.get(13u8)?.into_input();
    radio_interrupt.set_async_interrupt(rppal::gpio::Trigger::RisingEdge, move |_| {
        println!("Received Data");
        robot_relay_node.update();
    })?;

    // Add the processes to the executor
    let mut executor = SimpleMultiExecutor::new_with(
        vec![
            ("Cpu Relay Thread", &mut cpu_relay_node),
            ("Timeout Checker Thread", &mut timeout_checker),
        ]
    );

    // Run the processes until ctrl-c received
    executor.start();
    executor.update_loop();

    Ok(())
}
