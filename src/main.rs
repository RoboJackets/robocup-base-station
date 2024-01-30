//!
//! The basic principle of the base station is to have two NComm nodes that do the following:
//! 
//! 1. Receive commands from the Field Computer and forward said commands to the robots
//! 2. Receive information from the Robots and forward alive robot information to the base computer
//! 
//! We will also be using 2 sx127 radios.
//! 

use std::error::Error;

use ncomm::executor::{Executor, simple_multi_executor::SimpleMultiExecutor};

use robocup_base_station::one_radio::radio_node::RadioNode;
use robocup_base_station::timeout_checker::TimeoutCheckerNode;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use robojackets_robocup_rtp::Team;

use clap::Parser;

/// The Arguments passed to the base station program.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The address of the base computer udp socket to publish robot status messages to
    #[arg(required = true)]
    pub base_computer_address: String,

    // The port to send robot statuses to
    #[arg(required = true)]
    pub base_computer_status_port: u16,

    // The port to send the alive robots message to
    #[arg(required = true)]
    pub base_computer_alive_port: u16,

    // The address on the raspberry pi computer to bind the udp socket that is 
    // listening to incoming data from the base computer
    #[arg(default_value_t = String::from("0.0.0.0:8000"))]
    pub receive_bind_address: String,
    
    // The address on the raspberry pi computer to bind the udp socket that is
    // sending data to the base computer
    #[arg(default_value_t = String::from("0.0.0.0:8001"))]
    pub send_bind_address: String,

    // The address on this computer to bind the timeout checker to
    #[arg(default_value_t = String::from("0.0.0.0:8002"))]
    pub timeout_bind_address: String,

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

    // The maximum timeout between sends to the robot
    #[arg(default_value_t = 5)]
    pub send_timeout_ms: u128,

    // The length in milliseconds of the timeout before we consider a robot dead
    #[arg(short, long, default_value_t = 500)]
    pub timeout: u128,

    // The number of radios used by the base-station to communicate with the robots
    #[arg(short, long, default_value_t = false)]
    pub two_radios: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Parse the team
    let team = match (args.blue, args.yellow) {
        (true, _) | (false, false) => Team::Blue,
        (false, true) => Team::Yellow,
    };

    if args.two_radios {
        unimplemented!();
    } else {
        // Acquire the peripherals
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
        let gpio = Gpio::new()?;
        let csn = gpio.get(10)?.into_output();
        let ce = gpio.get(22)?.into_output();
        let delay = Delay::new();

        let publisher_send_address = format!("{}:{}", args.base_computer_address, args.base_computer_status_port);
        let mut radio_node = RadioNode::new(
            team,
            args.robots,
            args.send_timeout_ms,
            ce,
            csn,
            spi,
            delay,
            &args.send_bind_address,
            &publisher_send_address,
            &args.receive_bind_address
        );

        let receive_message_subscriber = radio_node.create_subscriber();
        let timeout_send_address = format!("{}:{}", args.base_computer_address, args.base_computer_alive_port);
        let mut timeout_node = TimeoutCheckerNode::new(
            args.robots,
            args.timeout,
            &args.timeout_bind_address,
            &timeout_send_address,
            receive_message_subscriber,
        );

        let mut executor = SimpleMultiExecutor::new_with(vec![
            ("Radio", &mut radio_node),
            ("Timeout", &mut timeout_node),
        ]);

        executor.start();

        executor.update_loop();
    }

    Ok(())
}
