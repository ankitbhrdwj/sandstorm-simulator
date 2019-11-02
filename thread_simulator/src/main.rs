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

extern crate rand;
extern crate threadpool;

use rand::Rng;
use threadpool::ThreadPool;
use std::sync::{Arc, Mutex};

mod setup;

fn main() {
    let n_threads:usize = 10000;

    let immut_map = Arc::new(Mutex::new(setup::setup_map()));

    let pool = ThreadPool::new(n_threads);
    
    for _ in 0..n_threads {
        let db = immut_map.clone();
        pool.execute(move || {
            let mut i = 0;
            loop {
                println!("Running at {}", i);
                i += 1;
                let mut rng = rand::thread_rng();
                let db = db.lock().unwrap();
                match db.get(&rng.gen_range(0, u64::max_value())) {
                    Some(v) => println!("Found {}", v),
                    None => println!("No dice"),
                }
            }
        });
    }
    pool.join();

}
