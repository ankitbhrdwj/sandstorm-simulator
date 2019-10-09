/* Copyright (c) 2019 University of Utah
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR(S) DISCLAIM ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL AUTHORS BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

extern crate client;
extern crate core_affinity;

use client::config::ClientConfig;
use client::cycles;

use std::mem::transmute;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;

struct Sender {
    // Socket to send the packets.
    socket: Arc<UdpSocket>,

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
}

impl Sender {
    fn new(socket: Arc<UdpSocket>, config: &ClientConfig) -> Sender {
        Sender {
            socket: socket,
            requests: config.num_reqs,
            sent: 0,
            rate_inv: cycles::cycles_per_second() / config.req_rate as u64,
            start: cycles::rdtsc(),
            next: 0,
        }
    }

    fn send(&mut self) {
        let mut buf = [0; 8];
        loop {
            if self.requests <= self.sent {
                return;
            }

            let curr: u64 = cycles::rdtsc();
            if curr >= self.next || self.next == 0 {
                unsafe {
                    buf[0..8].copy_from_slice(&{ transmute::<u64, [u8; 8]>(curr.to_le()) });
                }

                // Pick a random port to send the request to a random tenant.
                self.socket
                    .send_to(&buf, "127.0.0.1:1024")
                    .expect("couldn't send data");

                // Update the time stamp at which the next request should be generated, assuming that
                // the first request was sent out at self.start.
                self.sent += 1;
                self.next = self.start + self.sent * self.rate_inv;
            }
        }
    }
}

struct Receiver {
    // The network socket required to receives response packets from the network.
    socket: Arc<UdpSocket>,

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

impl Receiver {
    fn new(socket: Arc<UdpSocket>, config: &ClientConfig, master: bool) -> Receiver {
        Receiver {
            socket: socket,
            responses: config.num_resps,
            start: cycles::rdtsc(),
            recvd: 0,
            latencies: Vec::with_capacity(config.num_resps as usize),
            master: master,
            stop: 0,
        }
    }

    fn recv(&mut self) {
        let mut buf = [0; 8];
        loop {
            // Receieved maximum number of packets, exit now.
            if self.responses <= self.recvd {
                return;
            }

            // Check the responses; add latency to the vector.
            match self.socket.recv(&mut buf) {
                Ok(_received) => {
                    self.recvd += 1;
                    let timestamp = u64::from_be_bytes(buf);

                    // Take latency measurement after warmup; say after 2M responses.
                    if self.recvd > 2 * 1000 * 1000 && self.master {
                        self.latencies.push(cycles::rdtsc() - timestamp);
                        if self.recvd % 1000000 == 0 {
                            println!("Recvd {} responses", self.recvd);
                        }
                    }
                }
                Err(e) => println!("recv function failed: {:?}", e),
            }

            // Update the stop timestamp, if received the required number of responses.
            if self.responses <= self.recvd {
                self.stop = cycles::rdtsc();
            }
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        // Calculate & print the throughput for all client threads.
        println!(
            "Throughput {}",
            self.recvd as f64 / cycles::to_seconds(self.stop - self.start)
        );

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
                ">>> {} {}",
                cycles::to_seconds(m) * 1e9,
                cycles::to_seconds(t) * 1e9
            );
        }
    }
}

fn setup_send(socket: Arc<UdpSocket>, config: &ClientConfig) {
    Sender::new(socket, config).send();
}

fn setup_recv(socket: Arc<UdpSocket>, config: &ClientConfig, master: bool) {
    Receiver::new(socket, config, master).recv();
}

// This is the `main` thread
fn main() {
    let core_ids = core_affinity::get_core_ids().unwrap();
    assert_eq!(core_ids.len() % 2, 0);
    let mut start_port: u16 = 49000;

    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

    // The latencies are printed only for the master thread.
    let mut master = false;

    let mut i = 0;
    while i < core_ids.len() {
        let id = core_ids[i];

        let config = ClientConfig::load();
        start_port += i as u16;
        let ipaddr: IpAddr = config.server_ip.parse().unwrap();
        let addr = SocketAddr::new(ipaddr, start_port);
        let socket = Arc::new(UdpSocket::bind(addr).expect("couldn't bind to address"));
        let socket_clone = Arc::clone(&socket);

        // Alternative sender and receivers.
        children.push(thread::spawn(move || {
            core_affinity::set_for_current(id);
            setup_send(Arc::clone(&socket), &ClientConfig::load());
        }));
        i += 1;

        let id = core_ids[i];
        if i == (core_ids.len() - 1) {
            master = true;
        }

        children.push(thread::spawn(move || {
            core_affinity::set_for_current(id);
            setup_recv(Arc::clone(&socket_clone), &ClientConfig::load(), master);
        }));
        i += 1;
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}
