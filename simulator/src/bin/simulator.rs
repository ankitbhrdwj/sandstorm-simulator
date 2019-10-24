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

fn main() {
    env_logger::init();
    let config = simulator::config::Config::load();

    let mut cores = Vec::with_capacity(config.max_cores as usize);

    // Intialize Cores.
    for i in 0..config.max_cores {
        cores.push(simulator::cores::Core::new(i as u8, &config));
    }
    info!("Initialize {} cores", config.max_cores);

    loop {
        // Run each core one by one.
        for c in 0..config.max_cores {
            cores[c as usize].run();
        }

        // Check exit condition after each iteration.
        let mut exit = true;
        for c in 0..config.max_cores {
            if config.num_resps > cores[c as usize].request_processed {
                exit = false;
            }
        }
        if exit == true {
            info!("Request generation completed !!!\n");
            return;
        }
    }
}
