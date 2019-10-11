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

use std::net::{IpAddr, SocketAddr, UdpSocket};

/// This function listen on a UDP port and echo the content back to the source.
///
/// # Arguments
/// * `ip_address`: The IP Address which the process to use for binding to the socket.
/// * `port`: The UDP port which the process to use for binding to the socket.
pub fn echo(ip_address: IpAddr, port: u16) {
    let addr = SocketAddr::new(ip_address, port);
    let socket = UdpSocket::bind(addr).expect("couldn't bind to address");
    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = [0; 8];
    loop {
        let (_amt, src) = socket
            .recv_from(&mut buf)
            .expect("couldn't bind to address");

        socket.send_to(&buf, &src).expect("couldn't bind to address");
    }
}
