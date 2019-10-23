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

#![feature(asm, integer_atomics, atomic_min_max)]

extern crate serde;
extern crate serde_aux;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
pub extern crate log;
extern crate zipf;

/// This module is used to read and parse the configuration file.
pub mod config;

/// This module contains the various constants which are used throughout the simulation.
pub mod consts;

/// This module simulate the functionality of a CPU core.
pub mod cores;

/// This module contains the functionality of a tenant.
pub mod tenant;

/// This module contains the functionality related to a tenant-request.
pub mod request;

/// This module is used to generate the requests for given number of tenants.
pub mod dispatcher;

/// This module contains the CPU cycles related functionality; rdtsc() etc.
pub mod cycles;
