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

use super::consts;
use super::cycles;
use super::request::Request;

pub struct Core {
    // The id of the core.
    pub core_id: u8,

    // This tenant is active on this core.
    pub active_tenant: Option<u16>,

    // The simulated time-stamp per core.
    pub rdtsc: u64,

    // The number of requests processed by this core;
    pub request_processed: u64,

    // The latency for each request.
    pub latencies: Vec<u64>,
}

impl Core {
    pub fn new(id: u8, responses: u64) -> Core {
        Core {
            core_id: id,
            active_tenant: None,
            rdtsc: 0,
            request_processed: 0,
            latencies: Vec::with_capacity(responses as usize),
        }
    }

    pub fn rdtsc(&self) -> u64 {
        self.rdtsc.clone()
    }

    fn context_switch(&mut self, tenant: u16) {
        self.active_tenant = Some(tenant);
        self.rdtsc +=
            ((cycles::cycles_per_second() as f64 / 1e6) * consts::CONTEXT_SWITCH_TIME) as u64;
    }

    pub fn process_request(&mut self, req: Request) {
        let tenant = req.get_tenant();
        if Some(tenant) != self.active_tenant {
            self.context_switch(tenant);
        }

        self.rdtsc += ((cycles::cycles_per_second() as f64 / 1e6) * consts::PROCESSING_TIME) as u64;
        let latency = req.run(self.rdtsc);
        self.latencies.push(latency);
        self.request_processed += 1;
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        // Calculate & print median & tail latency only on the master thread.
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
            "Throughput {} Median(ns) {} Tail(ns) {}",
            self.request_processed as f64 / cycles::to_seconds(self.rdtsc - 0),
            cycles::to_seconds(m) * 1e9,
            cycles::to_seconds(t) * 1e9
        );
    }
}
