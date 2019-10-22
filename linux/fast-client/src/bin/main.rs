extern crate client;
extern crate e2d2;

use e2d2::allocators::*;
use e2d2::interface::*;
use e2d2::scheduler::*;

use std::sync::Arc;
use std::mem::transmute;
use std::fmt::Display;

use client::config;
use client::*;

pub struct ClientSend {
    // Network stack required to actually send RPC requests out the network.
    sender: dispatch::Sender,
    count: u16,
}
impl ClientSend {
    pub fn new(config: &config::ClientConfig, port: CacheAligned<PortQueue>) -> ClientSend {
        ClientSend {
            sender: dispatch::Sender::new(config, port),
            count: 0,
        }
    }

    pub fn send(&mut self) {
        let mut buf = [0; 8];
        let curr = cycles::rdtsc();
        unsafe {
            buf[0..8].copy_from_slice(&{ transmute::<u64, [u8; 8]>(curr.to_le()) });
        }

        self.sender.send_request(1024, &buf, self.count);
        self.count += 1;
        std::thread::sleep(std::time::Duration::from_nanos(1000));
    }
}

// Executable trait allowing LongRecv to be scheduled by Netbricks.
impl Executable for ClientSend {
    // Called internally by Netbricks.
    fn execute(&mut self) {
        self.send();
    }

    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}

pub struct ClientRecv<T>
where
    T: PacketTx + PacketRx + Display + Clone + 'static,
{
    // The network stack required to receives RPC response packets from a network port.
    receiver: dispatch::Receiver<T>,
}

impl<T> ClientRecv<T>
where
    T: PacketTx + PacketRx + Display + Clone + 'static,
{
    pub fn new(port: T) -> ClientRecv<T> {
        ClientRecv {
            receiver: dispatch::Receiver::new(port)
        }
    }

    pub fn recv(&mut self) {
        // Try to receive packets from the network port.
        // If there are packets, sample the latency of the server.
        if let Some(mut packets) = self.receiver.recv_res() {
            while let Some(packet) = packets.pop() {
                packet.free_packet();
                println!("recvd");
            }
        }
    }
}

// Executable trait allowing LongRecv to be scheduled by Netbricks.
impl<T> Executable for ClientRecv<T>
where
    T: PacketTx + PacketRx + Display + Clone + 'static,
{
    // Called internally by Netbricks.
    fn execute(&mut self) {
        self.recv();
    }

    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}

fn setup_send<S>(
    config: &config::ClientConfig,
    ports: Vec<CacheAligned<PortQueue>>,
    scheduler: &mut S,
    _core: i32,
) where
    S: Scheduler + Sized,
{
    match scheduler.add_task(ClientSend::new(config, ports[0].clone())) {
        Ok(_) => {
            println!(
                "Successfully added ClientSend with tx queue {}.",
                ports[0].txq()
            );
        }

        Err(ref err) => {
            println!("Error while adding to Netbricks pipeline {}", err);
            std::process::exit(1);
        }
    }
}

fn setup_recv<S>(ports: Vec<CacheAligned<PortQueue>>, scheduler: &mut S, _core: i32)
where
    S: Scheduler + Sized,
{
    if ports.len() != 1 {
        println!("Client should be configured with exactly 1 port!");
        std::process::exit(1);
    }

    // Add the receiver to a netbricks pipeline.
    match scheduler.add_task(ClientRecv::new(ports[0].clone())) {
        Ok(_) => {
            println!(
                "Successfully added ClientRecv with rx queue {}.",
                ports[0].rxq()
            );
        }

        Err(ref err) => {
            println!("Error while adding to Netbricks pipeline {}", err);
            std::process::exit(1);
        }
    }
}

fn main() {
    let config = config::ClientConfig::load();

    // Based on the supplied client configuration, compute the amount of time it will take to send
    // out `num_reqs` requests at a rate of `req_rate` requests per second.
    let exec = config.num_reqs / config.req_rate;

    let mut net_context = setup::config_and_init_netbricks(&config);

    // Setup the client pipeline.
    net_context.start_schedulers();

    // The core id's which will run the sender and receiver threads.
    // XXX The following two arrays heavily depend on the set of cores
    // configured in setup.rs
    let senders = [0];
    let receive = [1];
    assert!((senders.len() == 1) && (receive.len() == 1));

    // Setup 1 senders, and 1 receivers.
    for i in 0..1 {
        // First, retrieve a tx-rx queue pair from Netbricks
        let port = net_context
            .rx_queues
            .get(&senders[i])
            .expect("Failed to retrieve network port!")
            .clone();

        // Setup the receive side.
        net_context
            .add_pipeline_to_core(
                receive[i],
                Arc::new(
                    move |_ports, sched: &mut StandaloneScheduler, core: i32, _sibling| {
                        setup_recv(port.clone(), sched, core)
                    },
                ),
            )
            .expect("Failed to initialize receive side.");

        // Setup the send side.
        net_context
            .add_pipeline_to_core(
                senders[i],
                Arc::new(
                    move |ports, sched: &mut StandaloneScheduler, core: i32, _sibling| {
                        setup_send(&config::ClientConfig::load(), ports, sched, core)
                    },
                ),
            )
            .expect("Failed to initialize send side.");
    }

    // Allow the system to bootup fully.
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Run the client.
    net_context.execute();

    // Sleep for an amount of time approximately equal to the estimated execution time, and then
    // shutdown the client.
    std::thread::sleep(std::time::Duration::from_secs(exec as u64 + 11));

    // Stop the client.
    net_context.stop();
}
