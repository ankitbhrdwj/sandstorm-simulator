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
use super::cores::CoreType;
use super::request::Request;
use super::sched::Scheduler;

use std::collections::VecDeque;

pub struct Minos {
    // Task runqueue for small requests.
    pub small_rq: VecDeque<Box<Request>>,

    // Task runqueue for large request.
    pub large_rq: VecDeque<Box<Request>>,
}

impl Minos {
    pub fn new() -> Minos {
        Minos {
            small_rq: VecDeque::with_capacity(32),
            large_rq: VecDeque::with_capacity(32),
        }
    }
}

impl Scheduler for Minos {
    // Lookup the `Scheduler` trait for documentation on this method.
    fn create_task(&mut self, rdtsc: u64, task_time: f64, tenant_id: u16) {
        let req = Box::new(Request::new(tenant_id, rdtsc, task_time));
        if task_time == consts::TASK_DISTRIBUTION_TIME[0] {
            self.small_rq.push_back(req);
        } else {
            self.large_rq.push_back(req);
        }
    }

    // Lookup the `Scheduler` trait for documentation on this method.
    fn pick_next_task(&mut self, coretype: CoreType) -> Option<Box<Request>> {
        match coretype {
            CoreType::Small => {
                // Don't allow large tasks on small cores due to head of line blocking.
                self.small_rq.pop_front()
            }

            CoreType::Large => {
                // Small tasks on large cores are fine as there is no head of line blocking.
                if self.large_rq.len() > 0 {
                    self.large_rq.pop_front()
                } else {
                    self.small_rq.pop_front()
                }
            }
        }
    }

    // Lookup the `Scheduler` trait for documentation on this method.
    fn enqueue_task(&mut self, req: Box<Request>) {
        self.large_rq.push_front(req);
    }
}
