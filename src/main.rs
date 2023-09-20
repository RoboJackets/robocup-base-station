//!
//! The basic principle of the base station is to have two NComm nodes that do the following:
//! 
//! 1. Receive commands from the Field Computer and forward said commands to the robots
//! 2. Receive information from the Robots and forward alive robot information to the base computer
//! 
//! We will also be using 2 sx127 radios.
//! 

use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};

use ncomm::executor::{Executor, simple_multi_executor::SimpleMultiExecutor};

pub mod cpu_relay_node;
use cpu_relay_node::CpuRelayNode;

pub mod robot_relay_node;
use robot_relay_node::RobotRelayNode;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use sx127::LoRa;

pub const CPU_RELAY_BIND_ADDRESS: &str = "0.0.0.0:8000";
pub const ROBOT_RELAY_BIND_ADDRESS: &str = "0.0.0.0:8001";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;
    let gpio = Gpio::new()?;
    let cs = gpio.get(0u8)?.into_output();
    let reset = gpio.get(1u8)?.into_output();
    let delay = Delay::new();

    let radio = LoRa::new(spi, cs, reset, 8_000_000, delay).unwrap();
    let radio = Arc::new(Mutex::new(radio));

    let mut cpu_relay_node = CpuRelayNode::new(
        CPU_RELAY_BIND_ADDRESS,
        radio.clone(),
    );

    let mut robot_relay_node = RobotRelayNode::new(
        ROBOT_RELAY_BIND_ADDRESS,
        vec![
            args.get(0)
                .expect("Please provide the Base Computer Listening Address as the first argument")
                .as_str()
        ],
        radio.clone(),
    );

    let mut executor = SimpleMultiExecutor::new_with(
        vec![
            ("Cpu Relay Thread", &mut cpu_relay_node),
            ("Robot Relay Thread", &mut robot_relay_node)
        ]
    );

    executor.start();
    executor.update_loop();

    Ok(())
}
