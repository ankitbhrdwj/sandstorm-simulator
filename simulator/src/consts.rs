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

//====================================================================================================================//
// In CPU Cycles; Taken from Splinter v1.
pub const NOISOLATION_TENANT_SWITCH_CYCLES: u64 = 0;

// In CPU Cycles; taken from lmbench experiment.(Linux context switch + hint to the scheduler);
pub const PAGING_TENANT_SWITCH_CYCLES: u64 = 3500;

// In CPU Cycles; taken from HODOR paper.
pub const MPK_TENANT_SWITCH_CYCLES: u64 = 250;

// In CPU Cycles; taken from Shinjuku paper.(sysenter-sysexit + VMFunc + No Mask Switch).
pub const VMFUNC_TENANT_SWITCH_CYCLES: u64 = 450;

//====================================================================================================================//
// In CPU cycles. Shinjuku: 4900 to send-recieve signal(table 1) and 700 to swap context.
pub const NOISOLATION_PREEMPTION_OVERHEAD_CYCLES: u64 = 5600;

// In CPU cycles. Shinjuku: 4900 to send-recieve signal(table 1) and 2900 to swap context.
pub const PAGING_PREEMPTION_OVERHEAD_CYCLES: u64 = 7800;

// In CPU cycles. Shinjuku: 4900 to send-recieve signal(table 1), 250 to trampoline switch
// and 700 to swap context.
pub const MPK_PREEMPTION_OVERHEAD_CYCLES: u64 = 5850;

// In CPU cycles. Shinjuku: 2000 to send-recieve IPI(table 1) and 450 for
// VMFUNC_TENANT_SWITCH_CYCLES.
pub const VMFUNC_PREEMPTION_OVERHEAD_CYCLES: u64 = 2650;

//====================================================================================================================//
//Batch-size for each tenant
pub const BATCH_SIZE: usize = 8;

// Scheduler time quanta in micro-seconds.
pub const QUANTA_TIME: f64 = 5.0;

// Time distribution for short-running and long-running tasks.
// Short-running tasks take 1 us and long running tasks take 1 ms.
pub const TASK_DISTRIBUTION_TIME: [f64; 2] = [1.0, 1.0];
