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

use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand::rngs::ThreadRng;

pub struct Dispatch {
    // Total number of requests to generate.
    num_requests: u64,

    // Tenant skew used for request generation.
    _tenant_skew: f64,

    // The number of requests generated so far.
    sent: u64,

    // The tenant random number generator.
    tenant_rng: Box<Uniform<u16>>,

    // Random number generator.
    rng: Box<ThreadRng>,
}

impl Dispatch {
    pub fn new(config: &Config) -> Dispatch {
        Dispatch {
            num_requests: config.num_reqs,
            _tenant_skew: config.tenant_skew,
            sent: 0,
            tenant_rng: Box::new(Uniform::from(1..(config.num_tenants + 1) as u16)),
            rng: Box::new(thread_rng()),
        }
    }

    pub fn generate_request(&mut self) -> Option<u16> {
        if self.sent <= self.num_requests {
            self.sent += 1;
            if self.sent % 1000000 == 0 {
                info!("Generated {} requests", self.sent);
            }
            Some(self.tenant_rng.sample(&mut *self.rng))
        } else {
            None
        }
    }
}
