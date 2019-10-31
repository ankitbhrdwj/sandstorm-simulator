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

pub trait Scheduler {
    /// This method creates a new task and adds that to the first run-queue.
    ///
    /// # Arguments
    /// `rdtsc`: The CPU time at which the task was created.
    /// `task_time`: The amount of CPU Cycles this task needs to complete.
    /// `tenant_id`: The tells the tenant for which this was created.
    fn create_task(&mut self, rdtsc: u64, task_time: f64, tenant_id: u16);

    /// This method picks the next task to execute on the CPU.
    ///
    /// # Arguments
    /// `rdtsc`: The current timestamp counter value; used in deciding which task to pick next.
    ///
    /// # Return
    /// Return a task to execute on the current CPU.
    fn pick_next_task(&mut self, rdtsc: u64) -> Option<Box<Request>>;

    /// This method decides where to execute the task after preemption.
    ///
    /// # Argument
    /// `req`: The preempted task.
    fn enqueue_task(&mut self, req: Box<Request>);
}
