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

use super::config::Config;
use super::cycles;

use rand::distributions::Distribution;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use zipf::ZipfDistribution;

pub struct Dispatch {
    // Total number of requests to generate.
    num_requests: u64,

    // The number of requests generated so far.
    pub sent: u64,

    // The inverse of the rate at which requests are to be generated. Basically, the time interval
    // between two request generations in cycles.
    rate_inv: u64,

    // The time stamp at which the next request must be issued in cycles.
    next: u64,

    // The tenant random number generator.
    tenant_rng: Box<ZipfDistribution>,

    // Random number generator.
    rng: Box<ThreadRng>,

    // The lowest tenant number handled by a core.
    low: u16,

    // The highest tenant number handled by a core.
    high: u16,
}

impl Dispatch {
    pub fn new(config: &Config, low: u16, high: u16) -> Dispatch {
        Dispatch {
            num_requests: config.num_reqs,
            sent: 0,
            rate_inv: cycles::cycles_per_second() / config.req_rate as u64,
            next: 0,
            tenant_rng: Box::new(
                ZipfDistribution::new((high - low) as usize, config.tenant_skew)
                    .expect("Couldn't create tenant RNG."),
            ),
            rng: Box::new(thread_rng()),
            low: low,
            high: high,
        }
    }

    pub fn generate_request(&mut self, curr: u64) -> Option<u16> {
        if self.sent <= self.num_requests && (curr >= self.next || self.next == 0) {
            self.sent += 1;
            self.next = 0 + self.sent * self.rate_inv;
            let tenant = self.tenant_rng.sample(&mut *self.rng);
            if self.low + tenant as u16 <= self.high {
                Some(self.low + tenant as u16)
            } else {
                println!(
                    "The tenant number can't be higher than the max allowed tenant on this core"
                );
                None
            }
        } else {
            None
        }
    }

    pub fn get_next(&self) -> u64 {
        self.next.clone()
    }
}
