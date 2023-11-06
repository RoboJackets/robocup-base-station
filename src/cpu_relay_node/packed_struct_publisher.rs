//!
//! A Network Udp-Based Subscriber using PackedStruct
//! 
//! The UDP Subscriber receives data as a packed UDP Datagram
//! 

use std::net::UdpSocket;
use std::marker::PhantomData;

use packed_struct::PackedStruct;
use packed_struct::PackedStructSlice;
use packed_struct::types::bits::ByteArray;

use ncomm::publisher_subscriber::{SubscribeRemote, Receive};

pub struct PackedStructUdpSubscriber<Data: PackedStruct + Send + Clone, const DATA_SIZE: usize> {
    rx: UdpSocket,
    pub data: Vec<Data>,
}

impl<'a, Data: PackedStruct + Send + Clone, const DATA_SIZE: usize> PackedStructUdpSubscriber<Data, DATA_SIZE> {
    pub fn new(bind_address: &'a str, from_address: Option<&'a str>) -> Self {
        assert_eq!(Data::ByteArray::len(), DATA_SIZE);

        let socket = UdpSocket::bind(bind_address).expect("couldn't bind to the given address");
        if let Some(from_address) = from_address {
            socket.connect(from_address).expect("couldn't connect to the given address");
        }
        socket.set_nonblocking(true).unwrap();

        Self { rx: socket, data: Vec::new() }
    }
}

impl<Data: PackedStruct + Send + Clone, const DATA_SIZE: usize> Receive for PackedStructUdpSubscriber<Data, DATA_SIZE> {
    fn update_data(&mut self) {
        loop {
            let mut buf = [0u8; DATA_SIZE];
            match self.rx.recv(&mut buf) {
                Ok(_received_bytes) => {
                    println!("Found Data");
                    println!("{:?}", buf);
                    match Data::unpack_from_slice(&buf[..]) {
                        Ok(data) => self.data.push(data),
                        Err(_) => {
                            println!("Unable to decode data");
                            return;
                        }
                    }
                },
                Err(_) => break,
            }
        }
    }
}