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

use super::request::Request;
use super::sched::Scheduler;

use std::collections::VecDeque;

pub struct MultiQ {
    short_rq: VecDeque<Box<Request>>,
    long_rq: VecDeque<Box<Request>>,
    once_in_cycles: u64,
    next_long_tsc: u64,
}

impl MultiQ {
    pub fn new(once_in_cycles: f64) -> MultiQ {
        MultiQ {
            short_rq: VecDeque::with_capacity(32),
            long_rq: VecDeque::with_capacity(32),
            once_in_cycles: once_in_cycles as u64,
            next_long_tsc: once_in_cycles as u64,
        }
    }
}

impl Scheduler for MultiQ {
    // Lookup the `Scheduler` trait for documentation on this method.
    fn create_task(&mut self, rdtsc: u64, task_time: f64, tenant_id: u16) {
        let req = Box::new(Request::new(tenant_id, rdtsc, task_time));
        self.short_rq.push_back(req);
    }

    // Lookup the `Scheduler` trait for documentation on this method.
    fn pick_next_task(&mut self, rdtsc: u64) -> Option<Box<Request>> {
        if self.short_rq.len() == 0 || (rdtsc >= self.next_long_tsc && self.long_rq.len() > 0) {
            self.next_long_tsc += self.once_in_cycles;
            self.long_rq.pop_front()
        } else {
            self.short_rq.pop_front()
        }
    }

    // Lookup the `Scheduler` trait for documentation on this method.
    fn enqueue_task(&mut self, req: Box<Request>) {
        // Within the long running tasks do FCFS
        self.long_rq.push_front(req);
    }
}
