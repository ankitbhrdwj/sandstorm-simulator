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

use std::sync::Once;

static mut CYCLES_PER_SECOND: u64 = 0;
static INIT: Once = Once::new();

/// Perform once-only overall initialization for the cycles module, such
/// as calibrating the clock frequency.  This method is invoked automatically
/// during initialization.
/// Stolen from the RAMCloud code base. Thanks, John.
fn init() -> u64 {
    let cycles_per_second = 3.0 * 1e9;
    cycles_per_second as u64
}

/// Return the CPU cycles per second for the executing processor.
///
/// # Return
///
/// Number of CPU cycles per second.
pub fn cycles_per_second() -> u64 {
    unsafe {
        INIT.call_once(|| {
            CYCLES_PER_SECOND = init();
        });
        CYCLES_PER_SECOND
    }
}

pub fn cycles_per_us() -> f64 {
    cycles_per_second() as f64 / 1e6
}

/// Return a 64-bit timestamp using the rdtsc instruction.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn rdtsc() -> u64 {
    unsafe {
        let lo: u32;
        let hi: u32;
        asm!("rdtsc" : "={eax}"(lo), "={edx}"(hi) : : : "volatile");
        (((hi as u64) << 32) | lo as u64)
    }
}

/// Converts the number of CPU cycles to seconds.
///
/// # Arguments
/// *`cycles`: Number of CPU cycles.
///
/// # Return
/// Number of seconds corresponding to the given CPU cycles.
pub fn to_seconds(cycles: u64) -> f64 {
    cycles as f64 / cycles_per_second() as f64
}
