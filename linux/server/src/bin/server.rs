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

extern crate server;

use nix::sys::wait::wait;
use nix::unistd::{fork, ForkResult};
use server::listen::echo;
use std::net::IpAddr;

extern crate core_affinity;

/// This function forks a process.
///
/// # Arguments
/// * `ip_address`: The IP Address which the process to use for binding to the socket.
/// * `port`: The UDP port which the process to use for binding to the socket.
fn create_process(ip_address: IpAddr, port_num: u16, coreid: core_affinity::CoreId) {
    match fork() {
        Ok(ForkResult::Parent { child: _, .. }) => {}

        Ok(ForkResult::Child) => {
            core_affinity::set_for_current(coreid);
            echo(ip_address, port_num);
        }

        Err(_) => {
            println!("Fork failed");
        }
    }
}

fn main() {
    let mut port_num = 1024;
    let config = server::config::Config::load();
    let max_cores = config.max_cores;

    let core_ids = core_affinity::get_core_ids().unwrap();

    let ip_address: IpAddr = config.server_ip.parse().unwrap();
    let process_num = config.num_process;

    for i in 0..process_num {
        create_process(ip_address, port_num, core_ids[(i % max_cores) as usize]);
        port_num += 1;
    }
    println!(
        "The server forked {} processes on {} cores; processes are listening on {}-{} ports",
        process_num,
        max_cores,
        port_num - process_num as u16,
        port_num - 1
    );

    let _ = wait();
}
