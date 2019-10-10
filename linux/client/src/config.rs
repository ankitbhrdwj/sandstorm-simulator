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

use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClientConfig {
    // The IP Address for the client.
    pub client_ip: String,

    // The IP Address for the server.
    pub server_ip: String,

    // The number of teants the client will generate requests for.
    pub num_tenants: u64,

    // The number of requests that the client must generate.
    pub num_reqs: u64,

    // The number of responses that the client must receive before terminating the process.
    pub num_resps: u64,

    // The req rate per second.
    pub req_rate: u64,
}

impl ClientConfig {
    pub fn load() -> ClientConfig {
        let mut contents = String::new();
        let filename = "client.toml";

        let _ = File::open(filename).and_then(|mut file| file.read_to_string(&mut contents));

        match toml::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                panic!("Failure paring config file {}: {}", filename, e);
            }
        }
    }
}
