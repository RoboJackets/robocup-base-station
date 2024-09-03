//!
//! The basic principle of the base station is to have two NComm nodes that do the following:
//!
//! 1. Receive commands from the Field Computer and forward said commands to the robots
//! 2. Receive information from the Robots and forward alive robot information to the base computer
//!
//! We will also be using 2 sx127 radios.
//!
//! Communication with the Field Computer is as follows:
//! (field::8000 -> 0.0.0.0:8000) - Field Sends Control Commands
//! (0.0.0.0:8001 -> field::8001) - We Send Robot Statuses
//! (0.0.0.0:8002 -> field::8002) - We Send Alive Robots
//!

use std::{
    error::Error,
    sync::mpsc,
    thread::{self, spawn},
    time::Duration,
};

use ncomm::node::Node;

use robocup_base_station::one_radio::radio_node::RadioNode;
use robocup_base_station::timeout_checker::TimeoutCheckerNode;
use robocup_base_station::xbox_node::XboxControlNode;

use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SlaveSelect, Spi},
};

use robojackets_robocup_rtp::TEAM;

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

    // If true, a manual node (i.e. xbox-control node is spun up)
    #[arg(short, long, default_value_t = false)]
    pub manual: bool,

    // The number of radios used by the base-station to communicate with the robots
    #[arg(long, default_value_t = false)]
    pub two_radios: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.two_radios {
        unimplemented!();
    } else {
        // Acquire the peripherals
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
        let gpio = Gpio::new()?;
        let csn = gpio.get(8)?.into_output();
        let ce = gpio.get(22)?.into_output();
        let delay = Delay::new();

        let control_message_bind_address = format!("0.0.0.0:{}", args.control_message_port);
        // let control_message_send_address = format!("{}:{}", args.field_computer_address, args.control_message_port);
        let robot_status_bind_address = format!("0.0.0.0:{}", args.robot_status_port);
        let robot_status_send_address =
            format!("{}:{}", args.field_computer_address, args.robot_status_port);
        let alive_robots_bind_address = format!("0.0.0.0:{}", args.alive_robots_port);
        let alive_robots_send_address =
            format!("{}:{}", args.field_computer_address, args.alive_robots_port);

        let mut radio_node;
        let manual;
        if args.manual {
            let mut manual_node = XboxControlNode::new(TEAM);

            radio_node = RadioNode::new_manual(
                TEAM,
                args.robots,
                ce,
                csn,
                spi,
                delay,
                &control_message_bind_address,
                &robot_status_bind_address,
                &robot_status_send_address,
                manual_node.subscribe(),
            );
            manual = Some(manual_node);
        } else {
            radio_node = RadioNode::new(
                TEAM,
                args.robots,
                ce,
                csn,
                spi,
                delay,
                &control_message_bind_address,
                &robot_status_bind_address,
                &robot_status_send_address,
            );
            manual = None;
        }

        let receive_message_subscriber = radio_node.create_subscriber();
        let mut timeout_node = TimeoutCheckerNode::new(
            args.robots,
            args.timeout,
            Box::leak(Box::new(alive_robots_bind_address)),
            Box::leak(Box::new(alive_robots_send_address)),
            receive_message_subscriber,
        );

        let (radio_tx, radio_rx) = mpsc::channel();
        let (timeout_tx, timeout_rx) = mpsc::channel();
        let (manual_tx, manual_rx) = mpsc::channel();

        ctrlc::set_handler(move || {
            let _ = radio_tx.send(true);
            let _ = timeout_tx.send(true);
            let _ = manual_tx.send(true);
        })
        .expect("Unable to set ctrl-c handler");

        radio_node.start();
        timeout_node.start();

        let handle = spawn(move || {
            while let Err(_) = timeout_rx.try_recv() {
                timeout_node.update();
                thread::sleep(Duration::from_millis(args.timeout as u64));
            }
        });

        let manual_control_handle = if let Some(mut manual_control_node) = manual {
            Some(spawn(move || {
                while let Err(_) = manual_rx.try_recv() {
                    manual_control_node.update();
                    thread::sleep(Duration::from_millis(50));
                }
            }))
        } else {
            None
        };

        while let Err(_) = radio_rx.try_recv() {
            radio_node.update();
        }

        handle.join().unwrap();
        if let Some(manual_control_handle) = manual_control_handle {
            manual_control_handle.join().unwrap();
        }
    }

    Ok(())
}
