extern crate client;
extern crate e2d2;

use e2d2::allocators::*;
use e2d2::interface::*;
use e2d2::scheduler::*;

use std::fmt::Display;
use std::mem::transmute;
use std::sync::Arc;

use client::config;
use client::*;

use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand::rngs::ThreadRng;

pub struct ClientSend {
    // Network stack required to actually send RPC requests out the network.
    sender: dispatch::Sender,

    // Total number of requests to be sent out.
    requests: u64,

    // Number of requests that have been sent out so far.
    sent: u64,

    // The inverse of the rate at which requests are to be generated. Basically, the time interval
    // between two request generations in cycles.
    rate_inv: u64,

    // The time stamp at which the workload started generating requests in cycles.
    start: u64,

    // The time stamp at which the next request must be issued in cycles.
    next: u64,

    // The tenant random number generator.
    tenant_rng: Box<Uniform<u16>>,

    // Random number generator.
    rng: Box<ThreadRng>,
}
impl ClientSend {
    pub fn new(config: &config::ClientConfig, port: CacheAligned<PortQueue>) -> ClientSend {
        ClientSend {
            sender: dispatch::Sender::new(config, port),
            requests: config.num_reqs,
            sent: 0,
            rate_inv: cycles::cycles_per_second() / config.req_rate as u64,
            start: 0,
            next: 0,
            tenant_rng: Box::new(Uniform::from(1024..(1024 + config.num_tenants as u16))),
            rng: Box::new(thread_rng()),
        }
    }

    pub fn send(&mut self) {
        if self.requests <= self.sent {
            return;
        }

        let mut buf = [0; 8];
        let curr: u64 = cycles::rdtsc();
        while curr >= self.next || self.next == 0 {
            let timestamp: u64 = cycles::rdtsc();
            unsafe {
                buf[0..8].copy_from_slice(&{ transmute::<u64, [u8; 8]>(timestamp.to_le()) });
            }

            // Pick a random port to send the request to a random tenant.
            self.sender
                .send_request(self.tenant_rng.sample(&mut *self.rng), &buf, 0);

            // Update the time stamp at which the next request should be generated, assuming that
            // the first request was sent out at self.start.
            self.sent += 1;
            self.next = self.start + self.sent * self.rate_inv;
        }
    }
}

// Executable trait allowing LongRecv to be scheduled by Netbricks.
impl Executable for ClientSend {
    // Called internally by Netbricks.
    fn execute(&mut self) {
        if self.start == 0 {
            self.start = cycles::rdtsc();
        }
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

    // The number of response packets to wait for before printing out statistics.
    responses: u64,

    // Time stamp in cycles at which measurement started. Required to calculate observed
    // throughput of the Sandstorm server.
    start: u64,

    // The total number of responses received so far.
    recvd: u64,

    // Vector of sampled request latencies. Required to calculate distributions once all responses
    // have been received.
    latencies: Vec<u64>,

    // If true, this receiver will make latency measurements.
    master: bool,

    // Time stamp in cycles at which measurement stopped.
    stop: u64,
}

impl<T> ClientRecv<T>
where
    T: PacketTx + PacketRx + Display + Clone + 'static,
{
    pub fn new(config: &config::ClientConfig, port: T, master: bool) -> ClientRecv<T> {
        ClientRecv {
            receiver: dispatch::Receiver::new(port),
            responses: config.num_resps,
            start: 0,
            recvd: 0,
            latencies: Vec::with_capacity(config.num_resps as usize),
            master: master,
            stop: 0,
        }
    }

    pub fn recv(&mut self) {
        // Receieved maximum number of packets, exit now.
        if self.responses <= self.recvd {
            return;
        }

        // Try to receive packets from the network port.
        // If there are packets, sample the latency of the server.
        let mut buf = [0; 8];
        if let Some(mut packets) = self.receiver.recv_res() {
            while let Some(packet) = packets.pop() {
                self.recvd += 1;

                // Take latency after warmup.
                if self.recvd > 1 * 1000 * 1000 && self.master {
                    let payload = packet.get_payload();
                    if payload.len() == 18 {
                        let time = packet.get_payload().split_at(8).0;
                        buf.copy_from_slice(time);
                        let timestamp = u64::from_le_bytes(buf);
                        self.latencies.push(cycles::rdtsc() - timestamp);
                    } else {
                        println!("Malformed Response!!!");
                    }
                }

                // Free the packet.
                packet.free_packet();
            }
        }

        // Update the stop timestamp, if received the required number of responses.
        if self.responses <= self.recvd {
            self.stop = cycles::rdtsc();
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
        if self.start == 0 {
            self.start = cycles::rdtsc();
        }
        self.recv();
    }

    fn dependencies(&mut self) -> Vec<usize> {
        vec![]
    }
}

impl<T> Drop for ClientRecv<T>
where
    T: PacketTx + PacketRx + Display + Clone + 'static,
{
    fn drop(&mut self) {
        // Calculate & print median & tail latency only on the master thread.
        if self.master {
            self.latencies.sort();

            let m;
            let t = self.latencies[(self.latencies.len() * 99) / 100];
            match self.latencies.len() % 2 {
                0 => {
                    let n = self.latencies.len();
                    m = (self.latencies[n / 2] + self.latencies[(n / 2) + 1]) / 2;
                }

                _ => m = self.latencies[self.latencies.len() / 2],
            }

            println!(
                "Throughput {}, Median(ns) {}, Tail(ns) {}",
                self.recvd as f64 / cycles::to_seconds(self.stop - self.start),
                cycles::to_seconds(m) * 1e9,
                cycles::to_seconds(t) * 1e9,
            );
        }
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

fn setup_recv<S>(
    ports: Vec<CacheAligned<PortQueue>>,
    scheduler: &mut S,
    core: i32,
    config: &config::ClientConfig,
) where
    S: Scheduler + Sized,
{
    if ports.len() != 1 {
        println!("Client should be configured with exactly 1 port!");
        std::process::exit(1);
    }

    // Add the receiver to a netbricks pipeline.
    let mut master = false;
    if core == 1 {
        master = true;
    }
    match scheduler.add_task(ClientRecv::new(config, ports[0].clone(), master)) {
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
                        setup_recv(port.clone(), sched, core, &config::ClientConfig::load())
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
    std::thread::sleep(std::time::Duration::from_secs(exec as u64 + 5));

    // Stop the client.
    net_context.stop();
}
