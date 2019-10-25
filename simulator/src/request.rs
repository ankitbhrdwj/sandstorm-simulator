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

use super::{consts, cycles};

pub struct Request {
    // This task belong to tenant `tenant_id`.
    tenant_id: u16,

    // The starting time for this task.
    start_time: u64,

    // The task need `max_time` amount of micro-second time to complete.
    max_time: f64,

    // The remaining time in micro-second, which the task need to complete.
    remaining_time: f64,

    // The current state of the task.
    taskstate: TaskState,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskState {
    Runnable,
    Running,
    Preempted,
    Completed,
}

impl Request {
    pub fn new(tenant: u16, rdstc: u64, task_time: f64) -> Request {
        Request {
            tenant_id: tenant,
            start_time: rdstc,
            max_time: task_time,
            remaining_time: task_time,
            taskstate: TaskState::Runnable,
        }
    }

    pub fn run(&mut self) -> (u64, TaskState) {
        let mut time = 0;
        if self.remaining_time() <= consts::QUANTA_TIME {
            time += ((cycles::cycles_per_second() as f64 / 1e6) * self.remaining_time) as u64;
            self.taskstate = TaskState::Completed;
        } else {
            time += ((cycles::cycles_per_second() as f64 / 1e6) * consts::QUANTA_TIME) as u64;
            self.remaining_time -= consts::QUANTA_TIME;

            time += consts::PREEMPTION_OVERHEAD_CYCLES;
            self.taskstate = TaskState::Preempted;
        }
        (time, self.taskstate)
    }

    pub fn get_tenant(&self) -> u16 {
        self.tenant_id.clone()
    }

    pub fn start_time(&self) -> u64 {
        self.start_time.clone()
    }

    pub fn max_time(&self) -> f64 {
        self.max_time.clone()
    }

    pub fn remaining_time(&self) -> f64 {
        self.remaining_time.clone()
    }
}
