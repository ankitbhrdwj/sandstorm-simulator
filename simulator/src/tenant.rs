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

use std::collections::VecDeque;

pub struct Tenant {
    // Task runqueue for this tenant.
    pub rq: VecDeque<Box<Request>>,

    // The ID of the current tenant.
    pub tenant_id: u16,
}

impl Tenant {
    pub fn new(tenant: u16) -> Tenant {
        Tenant {
            rq: VecDeque::with_capacity(32),
            tenant_id: tenant,
        }
    }

    pub fn add_request(&mut self, rdtsc: u64, task_time: f64) {
        let req = Box::new(Request::new(self.tenant_id, rdtsc, task_time));
        self.rq.push_back(req);
    }

    pub fn get_request(&mut self) -> Option<Box<Request>> {
        self.rq.pop_front()
    }

    pub fn enqueue_task(&mut self, req: Box<Request>) {
        self.rq.push_back(req);
    }
}
