//!
//! Node that listens to inputs from the xbox controllers connected to the system and publishes these
//! commands to the robots
//!

use std::{
    fs::File, io::Read, sync::Arc
};

use ncomm::{
    node::Node,
    publisher_subscriber::{
        local::{LocalPublisher, MappedLocalSubscriber},
        Publish,
    },
};

use robojackets_robocup_rtp::{
    control_message::TriggerMode, ControlMessage, ControlMessageBuilder, Team,
};

/// The maximum velocity the robot can be moving in the X or Y direction (m/s)
pub const MAX_BODY_VELOCITY: f32 = 1.0;
/// The maximum velocity the robot can be turning in the w direction (rad/s)
pub const MAX_TURN_VELOCITY: f32 = 4.0;

const XPAD_PACKET_LENGTH: usize = 120;

/// Rust struct that is used to convert the inputs from controllers to usable inputs
struct XboxControlCommand {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub start: bool,
    pub select: bool,
    pub xbox_button: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    pub left_trigger: bool,
    pub right_trigger: bool,
    pub lstick_x: i8,
    pub lstick_y: i8,
    pub rstick_x: i8,
    pub rstick_y: i8,
    pub dpad_up: bool,
    pub dpad_right: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
}

impl From<&[u8; XPAD_PACKET_LENGTH]> for XboxControlCommand {
    fn from(value: &[u8; XPAD_PACKET_LENGTH]) -> Self {
        Self {
            a: value[4] == 1,
            b: value[12] == 1,
            x: value[20] == 1,
            y: value[28] == 1,
            start: value[60] == 1,
            select: value[52] == 1,
            xbox_button: value[68] == 1,
            left_shoulder: value[36] == 1,
            right_shoulder: value[44] == 1,
            left_trigger: false,
            right_trigger: false,
            lstick_x: 0,
            lstick_y: 0,
            rstick_x: 0,
            rstick_y: 0,
            dpad_up: value[108] == 1,
            dpad_right: value[100] == 1,
            dpad_down: value[116] == 1,
            dpad_left: value[92] == 1,
        }
    }
}

/// The Xbox Control Node looks for changes at /dev/input/js0 and /dev/input/js1 as inputs
/// from the xbox controllers connected to the base station.  It then takes these inputs
/// and publishes them to the node responsible for sending data to the robots.
///
/// The first Xbox Controller (/dev/input/js0) will be sending messages to robot 0
/// The second Xbox Controller (/dev/input/js1) will be sending messages to robot 1
pub struct XboxControlNode {
    team: Team,
    control_publisher: LocalPublisher<ControlMessage>,
}

impl XboxControlNode {
    /// Create a new XboxControlNode to handle inputs from the xbox controllers
    pub fn new(team: Team) -> Self {
        Self {
            team,
            control_publisher: LocalPublisher::new(),
        }
    }

    pub fn subscribe(&mut self) -> MappedLocalSubscriber<ControlMessage, u8> {
        self.control_publisher
            .create_mapped_subscriber(Arc::new(|message| *message.robot_id))
    }
}

impl Node for XboxControlNode {
    fn name(&self) -> String {
        String::from("Xbox-Control-Node")
    }

    fn get_update_delay(&self) -> u128 {
        50u128
    }

    fn start(&mut self) {
        let robot_0_message = ControlMessageBuilder::new()
            .team(self.team)
            .robot_id(0)
            .build();
        self.control_publisher.send(robot_0_message);
        let robot_1_message = ControlMessageBuilder::new()
            .team(self.team)
            .robot_id(1)
            .build();
        self.control_publisher.send(robot_1_message);
    }

    fn update(&mut self) {
        if let Ok(mut file) = File::open("/dev/input/js0") {
            let mut buffer = [0u8; XPAD_PACKET_LENGTH];
            if file.read(&mut buffer).is_ok() {
                let xbox_command = XboxControlCommand::from(&buffer);
                let control_message = ControlMessageBuilder::new()
                    .team(self.team)
                    .robot_id(0)
                    .body_x(if xbox_command.dpad_right {
                        MAX_BODY_VELOCITY
                    } else if xbox_command.dpad_left {
                        -MAX_BODY_VELOCITY
                    } else {
                        0.0
                    })
                    .body_y(if xbox_command.dpad_up {
                        MAX_BODY_VELOCITY
                    } else if xbox_command.dpad_down {
                        -MAX_BODY_VELOCITY
                    } else {
                        0.0
                    })
                    .body_w(if xbox_command.left_shoulder {
                        MAX_TURN_VELOCITY
                    } else if xbox_command.right_shoulder {
                        -MAX_TURN_VELOCITY
                    } else {
                        0.0
                    })
                    .dribbler_speed(if xbox_command.a { 1 } else { 0 })
                    .kick_strength(if xbox_command.x { 1 } else { 0 })
                    .trigger_mode(if xbox_command.x {
                        TriggerMode::Immediate
                    } else {
                        TriggerMode::StandDown
                    })
                    .build();
                self.control_publisher.send(control_message);
            }
        } else {
            let control_message = ControlMessageBuilder::new()
                .team(self.team)
                .robot_id(0)
                .build();
            self.control_publisher.send(control_message); 
        }

        if let Ok(mut file) = File::open("/dev/input/js1") {
            let mut buffer = [0u8; XPAD_PACKET_LENGTH];
            if file.read(&mut buffer).is_ok() {
                let xbox_command = XboxControlCommand::from(&buffer);
                let control_message = ControlMessageBuilder::new()
                    .team(self.team)
                    .robot_id(1)
                    .body_x(if xbox_command.dpad_right {
                        MAX_BODY_VELOCITY
                    } else if xbox_command.dpad_left {
                        -MAX_BODY_VELOCITY
                    } else {
                        0.0
                    })
                    .body_y(if xbox_command.dpad_up {
                        MAX_BODY_VELOCITY
                    } else if xbox_command.dpad_down {
                        -MAX_BODY_VELOCITY
                    } else {
                        0.0
                    })
                    .body_w(if xbox_command.left_shoulder {
                        MAX_TURN_VELOCITY
                    } else if xbox_command.right_shoulder {
                        -MAX_TURN_VELOCITY
                    } else {
                        0.0
                    })
                    .dribbler_speed(if xbox_command.a { 1 } else { 0 })
                    .kick_strength(if xbox_command.x { 1 } else { 0 })
                    .trigger_mode(if xbox_command.x {
                        TriggerMode::Immediate
                    } else {
                        TriggerMode::StandDown
                    })
                    .build();
                self.control_publisher.send(control_message);
            }
        } else {
            let control_message = ControlMessageBuilder::new()
                .team(self.team)
                .robot_id(1)
                .build();
            self.control_publisher.send(control_message);
        }
    }

    fn shutdown(&mut self) {
        let robot_0_message = ControlMessageBuilder::new()
            .team(self.team)
            .robot_id(0)
            .build();
        self.control_publisher.send(robot_0_message);
        let robot_1_message = ControlMessageBuilder::new()
            .team(self.team)
            .robot_id(1)
            .build();
        self.control_publisher.send(robot_1_message);
    }

    fn debug(&self) -> String {
        self.name()
    }
}
