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

use ncomm::executor::{Executor, simple_multi_executor::SimpleMultiExecutor};

pub mod cpu_relay_node;
use cpu_relay_node::CpuRelayNode;

pub mod robot_relay_node;
use robot_relay_node::RobotRelayNode;

pub const CPU_RELAY_BIND_ADDRESS: &str = "0.0.0.0:8000";
pub const ROBOT_RELAY_BIND_ADDRESS: &str = "0.0.0.0:8001";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut cpu_relay_node = CpuRelayNode::new(
        CPU_RELAY_BIND_ADDRESS,
    );
    let mut robot_relay_node = RobotRelayNode::new(
        ROBOT_RELAY_BIND_ADDRESS,
        vec![
            args.get(0)
                .expect("Please provide the Base Computer Listening Address as the first argument")
                .as_str()
        ]
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
