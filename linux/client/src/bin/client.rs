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

use client::cycles;

use std::mem::transmute;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;

struct Sender {
    socket: Arc<UdpSocket>,
}

impl Sender {
    fn new(socket: Arc<UdpSocket>) -> Sender {
        Sender { socket: socket }
    }

    fn send(&self) {
        let mut buf = [0; 8];
        loop {
            let curr: u64 = cycles::rdtsc();
            unsafe {
                buf[0..8].copy_from_slice(&{ transmute::<u64, [u8; 8]>(curr.to_le()) });
            }
            self.socket
                .send_to(&buf, "127.0.0.1:1024")
                .expect("couldn't send data");
        }
    }
}

struct Receiver {
    socket: Arc<UdpSocket>,
}

impl Receiver {
    fn new(socket: Arc<UdpSocket>) -> Receiver {
        Receiver { socket: socket }
    }

    fn recv(&self) {
        let mut buf = [0; 8];
        loop {
            match self.socket.recv(&mut buf) {
                Ok(_received) => {
                    let timestamp = u64::from_be_bytes(buf);
                    println!("{}", cycles::to_seconds(cycles::rdtsc() - timestamp) * 1e9);
                }
                Err(e) => println!("recv function failed: {:?}", e),
            }
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        println!("Drop");
    }
}

fn setup_send(socket: Arc<UdpSocket>) {
    Sender::new(socket).send();
}

fn setup_recv(socket: Arc<UdpSocket>) {
    Receiver::new(socket).recv();
}

// This is the `main` thread
fn main() {
    let core_ids = core_affinity::get_core_ids().unwrap();
    assert_eq!(core_ids.len() % 2, 0);
    let mut start_port: u16 = 49000;

    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

    let mut i = 0;
    while i < core_ids.len() {
        let id = core_ids[i];

        start_port += i as u16;
        let ipaddr: IpAddr = "127.0.0.1".parse().unwrap();
        let addr = SocketAddr::new(ipaddr, start_port);
        let socket = Arc::new(UdpSocket::bind(addr).expect("couldn't bind to address"));
        let socket_clone = Arc::clone(&socket);
        children.push(thread::spawn(move || {
            core_affinity::set_for_current(id);
            setup_send(Arc::clone(&socket));
        }));
        i += 1;

        // Alternative sender and receivers.
        let id = core_ids[i];
        children.push(thread::spawn(move || {
            core_affinity::set_for_current(id);
            setup_recv(Arc::clone(&socket_clone));
        }));
        i += 1;
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}
