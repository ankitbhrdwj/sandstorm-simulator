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

use super::config::{Config, Isolation};
use super::consts;
use super::cycles;
use super::dispatcher::Dispatch;
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

    // The dispather generates the requests for each core.
    pub dispatcher: Dispatch,

    // Starting tenant-id which this core handles.
    pub start_tenant: u16,

    // Last tenant-id which this core handles.
    pub end_tenant: u16,

    // Total number of context switches per core.
    pub num_context_switches: u64,

    // Isolation mechanism amoung domains on a core.
    pub isolation: Isolation,
}

impl Core {
    pub fn new(id: u8, config: &Config) -> Core {
        let uniform_divide: u16 = config.num_tenants as u16 / config.max_cores as u16;
        let low = (id as u16 * uniform_divide) + 1 as u16;
        let high = low + uniform_divide as u16;

        Core {
            core_id: id,
            active_tenant: None,
            rdtsc: 0,
            request_processed: 0,
            latencies: Vec::with_capacity(config.num_reqs as usize),
            dispatcher: Dispatch::new(config, low, high),
            start_tenant: low,
            end_tenant: high,
            num_context_switches: 0,
            isolation: config.isolation.clone(),
        }
    }

    pub fn rdtsc(&self) -> u64 {
        self.rdtsc.clone()
    }

    pub fn update_rdtsc(&mut self) {
        let next_dispatch_time = self.dispatcher.get_next();
        if self.rdtsc() < next_dispatch_time {
            self.rdtsc = next_dispatch_time;
        }
    }

    fn context_switch(&mut self, tenant: u16) {
        match self.isolation {
            Isolation::NoIsolation => {
                self.active_tenant = Some(tenant);
                self.num_context_switches += 1;
            }

            Isolation::PageTableIsolation => {
                self.active_tenant = Some(tenant);
                self.rdtsc += ((cycles::cycles_per_second() as f64 / 1e6)
                    * consts::CONTEXT_SWITCH_TIME) as u64;
                self.num_context_switches += 1;
            }

            Isolation::MpkIsolation => {
                info!("TODO: Implement Context Switch for {:?}", self.isolation);
            }

            Isolation::VmfuncIsolation => {
                info!("TODO: Implement Context Switch for {:?}", self.isolation);
            }
        }
    }

    pub fn generate_req(&mut self) -> Option<u16> {
        self.dispatcher.generate_request(self.rdtsc())
    }

    pub fn get_tenant_limit(&self) -> (u16, u16) {
        (self.start_tenant, self.end_tenant)
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

        if self.core_id == 0 && self.request_processed % 2000000 == 0 {
            info!("Processing requests");
        }
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
            "Throughput {:.2} Median(us) {:.2} Tail(us) {:.2} Context-Switches(%) {:.2} Execution-Time(sec) {:.2} CS-Time(sec) {:.2}, Total-Time(sec) {:.2}",
            self.request_processed as f64 / cycles::to_seconds(self.rdtsc - 0),
            cycles::to_seconds(m) * 1e6,
            cycles::to_seconds(t) * 1e6,
            (self.num_context_switches as f64 / self.request_processed as f64) * 100.0,
            self.request_processed as f64/ 1e6, (self.num_context_switches*2) as f64/1e6, cycles::to_seconds(self.rdtsc - 0)
        );
    }
}
