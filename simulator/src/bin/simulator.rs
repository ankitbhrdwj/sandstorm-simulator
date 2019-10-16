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

extern crate simulator;

use simulator::log::*;
use simulator::tenant::Tenant;

use std::collections::HashMap;

fn main() {
    env_logger::init();
    let config = simulator::config::Config::load();

    let mut cores = Vec::with_capacity(config.max_cores as usize);
    let mut tenants: HashMap<u16, Tenant> = HashMap::with_capacity(config.num_tenants as usize);

    // Intialize Cores.
    for i in 0..config.max_cores {
        cores.push(simulator::cores::Core::new(i as u8, &config));
    }
    info!("Initialize {} cores", config.max_cores);

    // Intialize Tenants.
    for i in 1..(config.num_tenants + 1) {
        tenants.insert(i as u16, Tenant::new(i as u16));
    }
    info!("Initialize {} Tenants\n", config.num_tenants);

    // Generate and process requests.
    loop {
        // Generate requests for different tenants
        for c in 0..config.max_cores {
            while let Some(tenant_id) = cores[c as usize].generate_req() {
                if let Some(tenant) = tenants.get_mut(&tenant_id) {
                    (*tenant)
                        .add_request(cores[(tenant_id % config.max_cores as u16) as usize].rdtsc());
                }
            }
        }

        // Process requests for each tenant one by one.
        for c in 0..config.max_cores {
            let mut t = c;
            while t <= config.num_tenants + 1 {
                if let Some(tenant) = tenants.get_mut(&(t as u16)) {
                    while let Some(request) = (*tenant).get_request() {
                        cores[c as usize].process_request(request);
                    }
                }
                t += config.max_cores;
            }
        }

        // Exit condition
        if config.num_resps <= cores[config.max_cores as usize - 1].request_processed {
            info!("Request generation completed !!!\n");
            return;
        }
    }
}
