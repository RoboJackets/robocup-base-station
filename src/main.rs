//!
//! The basic principle of the base station is to have two NComm nodes that do the following:
//! 
//! 1. Receive commands from the Field Computer and forward said commands to the robots
//! 2. Receive information from the Robots and forward alive robot information to the base computer
//! 
//! We will also be using 2 nRF24L01+ radios in the future.
//! 
//! Communication with the Field Computer is as follows:
//! (field::8000 -> 0.0.0.0:8000) - Field Sends Control Commands
//! (0.0.0.0:8001 -> field:8001) - We Send Robot Statuses
//! (0.0.0.0:8002 -> field:8002) - We Send Alive Robots
//! 

use std::error::Error;

use ncomm::prelude::*;
use ncomm::executors::ThreadedExecutor;

use robocup_base_station::{radio_node::RadioNode, timeout_checker::TimeoutCheckerNode, visor_node::VisorNode, NodeIdentifier};

use robojackets_robocup_rtp::Team;
use rppal::gpio::Gpio;

use crossbeam::channel::unbounded;

use clap::Parser;

/// The Arguments passed to the base station program.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The IPv4 Address of the Field Computer
    #[arg(default_value_t = String::from("10.42.0.1"))]
    pub field_computer_address: String,

    // Control Message Port
    #[arg(default_value_t = 8000)]
    pub control_message_port: u16,

    // Robot Status Port
    #[arg(default_value_t = 8001)]
    pub robot_status_port: u16,

    // Alive Robots Port
    #[arg(default_value_t = 8002)]
    pub alive_robots_port: u16,

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
    #[arg(long, default_value_t = false)]
    pub two_radios: bool,

    // Should we be running the blue team
    #[arg(short, long, default_value_t = true)]
    pub blue: bool,

    // Should we be running the yellow team
    #[arg(short, long, default_value_t = false)]
    pub yellow: bool,

    // Should we run the diagnostics node
    #[arg(short, long, default_value_t = false)]
    pub diagnostics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let team = if args.yellow {
        Team::Yellow
    } else {
        Team::Blue
    };

    let mut gpio = Gpio::new()?;

    let control_message_bind_address = format!("0.0.0.0:{}", args.control_message_port);
    let robot_status_bind_address = format!("0.0.0.0:{}", args.robot_status_port);
    let robot_status_send_address = format!("{}:{}", args.field_computer_address, args.robot_status_port);
    let alive_robots_bind_address = format!("0.0.0.0:{}", args.alive_robots_port);
    let alive_robots_send_address = format!("{}:{}", args.field_computer_address, args.alive_robots_port);

    let mut radio_node = RadioNode::new(
        team,
        0,
        args.robots,
        control_message_bind_address,
        robot_status_bind_address,
        robot_status_send_address,
        &mut gpio,
    );

    let timeout_node = TimeoutCheckerNode::new(
        args.robots,
        args.timeout,
        alive_robots_bind_address,
        alive_robots_send_address,
        radio_node.create_subscriber(),
    );

    let mut other_nodes: Vec<Box<dyn Node<NodeIdentifier>>> = Vec::new();
    if args.diagnostics {
        let (radio_diagnostics_subscriber, radio_update_subscriber) = radio_node.get_diagnostics();
        other_nodes.push(Box::new(VisorNode::new(radio_diagnostics_subscriber, radio_update_subscriber, args.robots.into())));
    }

    let (interrupt_tx, interrupt_rx) = unbounded();

    ctrlc::set_handler(move || {
        interrupt_tx.send(true).unwrap();
    }).expect("Unable to set ctrl-c handler");

    let mut executor = ThreadedExecutor::new_with(
        interrupt_rx,
        0,
        vec![
            (vec![Box::new(radio_node)], 1),
            (vec![Box::new(timeout_node)], 2),
            (other_nodes, 3),
        ]
    );

    executor.start();
    executor.update_loop();

    Ok(())
}
