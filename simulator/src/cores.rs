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

use super::config::{Config, Distribution as Dist, Isolation};
use super::consts;
use super::cycles;
use super::dispatcher::Dispatch;
use super::request::{Request, TaskState};
use super::rr_sched::RoundRobin;
use super::tenant::Tenant;

use std::cmp::min;
use std::ops::Range;
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;

use rand::distributions::weighted::alias_method::WeightedIndex;
use rand::distributions::Distribution;
use rand::prelude::*;
use rand::rngs::ThreadRng;

pub struct Simulator {
    config: Config,
    cores: Vec<Core>,
    latencies: Vec<u64>,
    tenants: HashMap<u64, Arc<RefCell<Tenant>>>,
}

impl Simulator {
    pub fn new() -> Simulator {
        let config = Config::load();
        info!("Starting the Simulator with config {:?}\n", config);
        let mut tenants = HashMap::with_capacity(config.num_tenants as usize);
        for i in 1..config.num_tenants + 1 {
            tenants.insert(i, Arc::new(RefCell::new(Tenant::new(i as u16, Box::new(RoundRobin::new())))));
        }
        let max_cores = config.max_cores as usize;
        let num_reqs = config.num_reqs as usize;

        Simulator {
            config: config,
            cores: Vec::with_capacity(max_cores),
            latencies: Vec::with_capacity(max_cores * num_reqs),
            tenants: tenants,
        }
    }

    pub fn core_init(&mut self) {
        for i in 0..self.config.max_cores {
            self.cores.push(Core::new(i as u8, &self.config, self.config.max_cores, &self.tenants));
        }
    }

    pub fn start(&mut self) {
        self.core_init();
        loop {
            // Run each core one by one.
            for c in 0..self.config.max_cores {
                self.cores[c as usize].run();
                let mut latency: Vec<u64> = self.cores[c as usize].latencies.drain(..).collect();
                self.latencies.append(&mut latency);
            }

            // Check exit condition after each iteration.
            let mut exit = true;
            for c in 0..self.config.max_cores {
                if self.config.num_resps > self.cores[c as usize].request_processed {
                    exit = false;
                }
            }
            if exit == true {
                info!("Request generation completed !!!\n");
                return;
            }
        }
    }
}

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

    // Total number of MPK switches per core.
    pub num_mpk_switches: u64,

    // Total number of MPK switches per core.
    pub num_vmfunc_switches: u64,

    // Total number of preemptions per core.
    pub num_preemptions: u64,

    // Isolation mechanism amoung domains on a core.
    pub isolation: Isolation,

    // Tenant vector, which holds the reference to tenants for a particular core.
    pub tenants: Vec<Arc<RefCell<Tenant>>>,

    // Batch size used by the core/scheduler.
    batch_size: usize,

    // Distribution mechanism amoung tenants on a core.
    pub distribution: Dist,

    // Range for MPK domains.
    pub mpk_domains: Vec<Range<u16>>,

    // Range for VMFunc domains.
    pub vmfunc_domains: Vec<Range<u16>>,

    // Outstanding tasks in the queue.
    outstanding: usize,

    // Distribution of short-running and long-running tasks.
    pub task_distribution: WeightedIndex<f64>,

    // Random number generator.
    rng: Box<ThreadRng>,

    // The last completed or preempted in the middle.
    last_task_state: TaskState,
}

impl Core {
    pub fn new(id: u8, config: &Config, num_cores: u64, tenants: &HashMap<u64, Arc<RefCell<Tenant>>>) -> Core {
        let uniform_divide: u16 = config.num_tenants as u16 / num_cores as u16;
        let low = (id as u16 * uniform_divide) + 1 as u16;
        let mut high = low + uniform_divide as u16;
        if id == num_cores as u8 - 1 {
            high = config.num_tenants as u16 + 1;
        }

        // Partition tenants in MPK Domains.
        let tenants_per_domain = 15;
        let num_domains = (high - low) / tenants_per_domain;
        let mut mpkdomains = Vec::with_capacity(num_domains as usize);

        let mut mpk_low = low;
        while mpk_low < high {
            let mpk_high = min(mpk_low + tenants_per_domain, high);
            mpkdomains.push(Range {
                start: mpk_low,
                end: mpk_high,
            });
            mpk_low = mpk_high;
        }

        // Partition tenants in VMFUNC Domains.
        let tenants_per_domain = 512;
        let num_domains = (high - low) / tenants_per_domain;
        let mut vmdomains = Vec::with_capacity(num_domains as usize);

        let mut vm_low = low;
        while vm_low < high {
            let vm_high = min(vm_low + tenants_per_domain, high);
            vmdomains.push(Range {
                start: vm_low,
                end: vm_high,
            });
            vm_low = vm_high;
        }

        // Intialize the tenants and assign these tenants to this core.
        let mut tenants_vec: Vec<Arc<RefCell<Tenant>>> = Vec::with_capacity((high - low) as usize);
        for i in low..high {
            let tenant = tenants.get(&(i as u64)).unwrap();
            tenants_vec.push(Arc::clone(tenant));
        }

        let mut batch_size = 1;
        if config.batching == true {
            batch_size = consts::BATCH_SIZE;
        }

        Core {
            core_id: id,
            active_tenant: None,
            rdtsc: 0,
            request_processed: 0,
            latencies: Vec::with_capacity(batch_size),
            dispatcher: Dispatch::new(config, low, high),
            start_tenant: low,
            end_tenant: high,
            num_context_switches: 0,
            num_mpk_switches: 0,
            num_vmfunc_switches: 0,
            num_preemptions: 0,
            isolation: config.isolation.clone(),
            tenants: tenants_vec,
            batch_size: batch_size,
            distribution: config.distribution.clone(),
            mpk_domains: mpkdomains,
            vmfunc_domains: vmdomains,
            outstanding: 0,
            task_distribution: WeightedIndex::new(vec![99.9, 0.1]).unwrap(),
            rng: Box::new(thread_rng()),
            last_task_state: TaskState::Completed,
        }
    }

    pub fn rdtsc(&self) -> u64 {
        self.rdtsc.clone()
    }

    pub fn update_rdtsc(&mut self) {
        let next_dispatch_time = self.dispatcher.get_next();
        if self.outstanding == 0 && self.rdtsc() < next_dispatch_time {
            self.rdtsc = next_dispatch_time;
        }
    }

    fn tenant_switch(&mut self, tenant: u16) {
        if self.last_task_state == TaskState::Preempted {
            self.active_tenant = Some(tenant);
            return;
        }

        match self.isolation {
            Isolation::NoIsolation => {
                self.active_tenant = Some(tenant);
                self.num_context_switches += 1;
            }

            Isolation::PageTableIsolation => {
                self.active_tenant = Some(tenant);
                self.rdtsc += consts::PAGING_TENANT_SWITCH_CYCLES;
                self.num_context_switches += 1;
            }

            Isolation::MpkIsolation => {
                // If this is not the starting of the scheduler, then some tenant must be active.
                // Otherwise, do a full context-switch to run the tenant process on this core.
                if let Some(curr_tenant) = self.active_tenant {
                    for range in &self.mpk_domains {
                        if range.contains(&curr_tenant) {
                            // If the new tenant is in same MPK Domain as old tenant then do the
                            // light-weight MPK domain switch; otherwise do full context-switch.
                            if tenant < range.end {
                                self.active_tenant = Some(tenant);
                                self.rdtsc += consts::MPK_TENANT_SWITCH_CYCLES;
                                self.num_mpk_switches += 1;
                            } else {
                                self.active_tenant = Some(tenant);
                                self.rdtsc += consts::PAGING_TENANT_SWITCH_CYCLES;
                                self.num_context_switches += 1;
                            }
                        }
                    }
                } else {
                    self.active_tenant = Some(tenant);
                    self.rdtsc += consts::PAGING_TENANT_SWITCH_CYCLES;
                    self.num_context_switches += 1;
                }
            }

            Isolation::VmfuncIsolation => {
                if let Some(curr_tenant) = self.active_tenant {
                    for range in &self.vmfunc_domains {
                        if range.contains(&curr_tenant) {
                            if tenant < range.end {
                                self.active_tenant = Some(tenant);
                                self.rdtsc += consts::VMFUNC_TENANT_SWITCH_CYCLES;
                                self.num_vmfunc_switches += 1;
                            } else {
                                self.active_tenant = Some(tenant);
                                self.rdtsc += consts::PAGING_TENANT_SWITCH_CYCLES;
                                self.num_context_switches += 1;
                            }
                        }
                    }
                } else {
                    self.active_tenant = Some(tenant);
                    self.rdtsc += consts::PAGING_TENANT_SWITCH_CYCLES;
                    self.num_context_switches += 1;
                }
            }
        }
    }

    pub fn generate_req(&mut self) -> Option<u16> {
        if let Some(t) = self.dispatcher.generate_request(self.rdtsc()) {
            let tenant;
            match self.distribution {
                Dist::Zipf => {
                    tenant = self.start_tenant + t - 1;
                    if tenant >= self.end_tenant {
                        None
                    } else {
                        Some(tenant)
                    }
                }

                Dist::Uniform => {
                    tenant = t;
                    if tenant >= self.end_tenant {
                        None
                    } else {
                        Some(tenant)
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn get_tenant_limit(&self) -> (u16, u16) {
        (self.start_tenant, self.end_tenant)
    }

    pub fn process_request(&mut self, mut req: Box<Request>, index: usize) {
        let tenant = req.get_tenant();
        if Some(tenant) != self.active_tenant {
            self.tenant_switch(tenant);
        }

        let (time, taskstate) = req.run(&self.isolation);
        self.rdtsc += time;
        match taskstate {
            TaskState::Completed => {
                let latency = self.rdtsc() - req.start_time();
                self.latencies.push(latency);
                self.request_processed += 1;
                self.outstanding -= 1;
                self.last_task_state = taskstate;
            }

            TaskState::Preempted => {
                self.num_preemptions += 1;
                self.tenants[index].borrow_mut().enqueue_task(req);
                self.last_task_state = taskstate;
            }

            TaskState::Runnable | TaskState::Running => {
                println!("The task shouldn't return this state");
            }
        }

        if self.core_id == 0 && self.request_processed % 2000000 == 0 {
            info!("Processing requests");
        }
    }

    fn run_dispatcher(&mut self) {
        while let Some(tenant_id) = self.generate_req() {
            let dindex = self.task_distribution.sample(&mut *self.rng);
            let task_time = consts::TASK_DISTRIBUTION_TIME[dindex];
            let index = tenant_id as usize - self.start_tenant as usize;
            self.tenants[index].borrow_mut().add_request(self.rdtsc, task_time);
            self.outstanding += 1;
        }
    }

    pub fn run(&mut self) {
        let (low, high) = self.get_tenant_limit();

        // Go through each tenant one by one; executing BATCH_SIZE tasks at a time.
        for t in low..high {
            let index: usize = (t - low) as usize;
            for _t in 0..self.batch_size {
                // Generate some more requests.
                self.run_dispatcher();

                let task = self.tenants[index].borrow_mut().get_request(self.rdtsc);
                if let Some(task) = task {
                    self.process_request(task, index);
                } else {
                    break;
                }
            }
        }

        // Update the timestamp counter
        self.update_rdtsc();
    }
}

impl Drop for Simulator {
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
            "Latency: Median(us) {:.2} Tail(us) {:.2}",
            cycles::to_seconds(m) * 1e6,
            cycles::to_seconds(t) * 1e6,
        );
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        let preemption_cycles;
        let cs_cycles;
        match self.isolation {
            Isolation::NoIsolation => {
                preemption_cycles =
                    self.num_preemptions * consts::NOISOLATION_PREEMPTION_OVERHEAD_CYCLES;
                cs_cycles = 0;
            }
            Isolation::PageTableIsolation => {
                preemption_cycles =
                    self.num_preemptions * consts::PAGING_PREEMPTION_OVERHEAD_CYCLES;
                cs_cycles = self.num_context_switches * consts::PAGING_TENANT_SWITCH_CYCLES;
            }
            Isolation::MpkIsolation => {
                preemption_cycles = self.num_preemptions * consts::MPK_PREEMPTION_OVERHEAD_CYCLES;
                cs_cycles = self.num_mpk_switches * consts::MPK_TENANT_SWITCH_CYCLES;
            }
            Isolation::VmfuncIsolation => {
                preemption_cycles =
                    self.num_preemptions * consts::VMFUNC_PREEMPTION_OVERHEAD_CYCLES;
                cs_cycles = self.num_vmfunc_switches * consts::VMFUNC_TENANT_SWITCH_CYCLES;
            }
        }

        println!(
            "Throughput {:.2} Context-Switches(%) {:.2} Execution-Time(sec) {:.2} CS-Time(sec) {:.2} Total-Time(sec) {:.2}",
            self.request_processed as f64 / cycles::to_seconds(self.rdtsc - 0),
            (self.num_context_switches as f64 / self.request_processed as f64) * 100.0,
            self.request_processed as f64/ 1e6,
            cycles::to_seconds(cs_cycles + preemption_cycles),
            cycles::to_seconds(self.rdtsc - 0)
        );
    }
}
