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

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

use super::e2d2::headers::*;
use super::toml;

/// To show the error while parsing the MAC address.
#[derive(Debug, Clone)]
pub struct ParseError;

impl Error for ParseError {
    fn description(&self) -> &str {
        "Malformed MAC address."
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Malformed MAC address.")
    }
}

/// Parses str into a MacAddress or returns ParseError.
/// str must be formatted six colon-separated hex literals.
pub fn parse_mac(mac: &str) -> Result<MacAddress, ParseError> {
    let bytes: Result<Vec<_>, _> = mac.split(':').map(|s| u8::from_str_radix(s, 16)).collect();

    match bytes {
        Ok(bytes) => {
            if bytes.len() == 6 {
                Ok(MacAddress::new_from_slice(&bytes))
            } else {
                Err(ParseError {})
            }
        }
        Err(_) => Err(ParseError {}),
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClientConfig {
    // The PCI address of the NIC the client is going to transmit and receive packets on.
    pub nic_pci: String,

    client_mac: String,
    server_mac: String,

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

    /// Parse `mac_address` into NetBrick's format or panic if malformed.
    /// Linear time, so ideally we'd store this in ClientConfig, but TOML parsing makes that tricky.
    pub fn parse_mac(&self) -> MacAddress {
        parse_mac(&self.client_mac)
            .expect("Missing or malformed mac_address field in client config.")
    }

    /// Parse `server_mac_address` into NetBrick's format or panic if malformed.
    /// Linear time, so ideally we'd store this in ClientConfig, but TOML parsing makes that tricky.
    pub fn parse_server_mac(&self) -> MacAddress {
        parse_mac(&self.server_mac)
            .expect("Missing or malformed server_mac_address field in client config.")
    }
}
