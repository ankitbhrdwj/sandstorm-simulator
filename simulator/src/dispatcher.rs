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

use super::config;
use super::cycles;

use rand::distributions::{Distribution, Uniform};
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

    // The tenant zipf number generator.
    tenant_rng_zipf: Box<ZipfDistribution>,

    // The tenant random number generator.
    tenant_rng_uniform: Box<Uniform<u16>>,

    // Random number generator.
    rng: Box<ThreadRng>,

    // Distribution mechanism amoung tenants on a core.
    distribution: config::Distribution,
}

impl Dispatch {
    pub fn new(
        config: &config::Config,
        low: u16,
        high: u16,
        req_rate: u64,
        num_reqs: u64,
    ) -> Dispatch {
        let num_tenants = (high - low) as usize;
        Dispatch {
            num_requests: num_reqs,
            sent: 0,
            rate_inv: cycles::cycles_per_second() / req_rate,
            next: 0,
            tenant_rng_zipf: Box::new(
                ZipfDistribution::new(num_tenants, config.tenant_skew)
                    .expect("Couldn't create tenant RNG."),
            ),
            tenant_rng_uniform: Box::new(Uniform::from(low..high)),
            rng: Box::new(thread_rng()),
            distribution: config.distribution.clone(),
        }
    }

    pub fn generate_request(&mut self, curr: u64) -> Option<u16> {
        if self.sent <= self.num_requests && (curr >= self.next || self.next == 0) {
            self.sent += 1;
            self.next = 0 + self.sent * self.rate_inv;
            match self.distribution {
                config::Distribution::Uniform => {
                    Some(self.tenant_rng_uniform.sample(&mut *self.rng))
                }

                config::Distribution::Zipf => {
                    Some(self.tenant_rng_zipf.sample(&mut *self.rng) as u16)
                }
            }
        } else {
            None
        }
    }

    pub fn get_next(&self) -> u64 {
        self.next.clone()
    }
}
