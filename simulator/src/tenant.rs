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

pub struct Tenant {
    /// The scheduler is used to determine the schedule for the tasks for current tenant.
    pub sched: Box<dyn Scheduler>,

    // The ID of the current tenant.
    pub tenant_id: u16,
}

impl Tenant {
    pub fn new(tenant: u16, policy: Box<dyn Scheduler>) -> Tenant {
        Tenant {
            sched: policy,
            tenant_id: tenant,
        }
    }

    pub fn add_request(&mut self, rdtsc: u64, task_time: f64) {
        self.sched.create_task(rdtsc, task_time, self.tenant_id);
    }

    pub fn get_request(&mut self, rdtsc: u64) -> Option<Box<Request>> {
        self.sched.pick_next_task(rdtsc)
    }

    pub fn enqueue_task(&mut self, req: Box<Request>) {
        self.sched.enqueue_task(req);
    }
}
