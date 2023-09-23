//!
//! The basic principle of the base station is to have two NComm nodes that do the following:
//! 
//! 1. Receive commands from the Field Computer and forward said commands to the robots
//! 2. Receive information from the Robots and forward alive robot information to the base computer
//! 
//! We will also be using 2 sx127 radios.
//! 

use std::error::Error;
use std::sync::{Arc, Mutex};

use ncomm::executor::{Executor, simple_multi_executor::SimpleMultiExecutor};

pub mod cpu_relay_node;
use cpu_relay_node::CpuRelayNode;

pub mod robot_relay_node;
use robot_relay_node::RobotRelayNode;

pub mod timeout_checker;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use rtp::Team;
use sx127::LoRa;

use clap::Parser;
use timeout_checker::TimeoutCheckerNode;

/// The Arguments passed to the base station program.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The address on the raspberry pi computer to bind the udp socket that is 
    // listening to incoming data from the base computer
    #[arg(default_value_t = String::from("0.0.0.0:8000"))]
    pub receive_bind_address: String,
    
    // The address on the raspberry pi computer to bind the udp socket that is
    // sending data to the base computer
    #[arg(default_value_t = String::from("0.0.0.0:8001"))]
    pub send_bind_address: String,

    // The address of the base computer udp socket to publish robot status messages to
    #[arg(required = true)]
    pub base_computer_address: String,

    // Boolean value to determine whether the team this base station is sending commands to
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
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;
    let gpio = Gpio::new()?;
    let cs = gpio.get(0u8)?.into_output();
    let reset = gpio.get(1u8)?.into_output();
    let delay = Delay::new();

    // Create Radio
    let radio = LoRa::new(spi, cs, reset, 8_000_000, delay).unwrap();
    let radio = Arc::new(Mutex::new(radio));

    // Create the process that receives commands from the base computer and relays such commands to the robots
    let mut cpu_relay_node = CpuRelayNode::new(
        args.receive_bind_address.as_str(),
        radio.clone(),
        team,
    );

    // Create the process that receives status messages from the robots and relays that information to the base computer
    let mut robot_relay_node = RobotRelayNode::new(
        args.send_bind_address.as_str(),
        vec![
            args.base_computer_address.as_str(),
        ],
        radio.clone(),
        team,
        args.robots,
    );

    // Create the process that keeps up to date with reviving and sleeping the robots
    let mut timeout_checker = TimeoutCheckerNode::new(
        radio,
        robot_relay_node.create_subscriber(),
        team,
        args.robots,
        args.timeout,
    );

    // Add the processes to the executor
    let mut executor = SimpleMultiExecutor::new_with(
        vec![
            ("Cpu Relay Thread", &mut cpu_relay_node),
            ("Robot Relay Thread", &mut robot_relay_node),
            ("Timeout Checker Thread", &mut timeout_checker),
        ]
    );

    // Run the processes until ctrl-c received
    executor.start();
    executor.update_loop();

    Ok(())
}
