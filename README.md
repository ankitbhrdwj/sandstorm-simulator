# Sandstorm-simulator ![Rust](https://github.com/ankit-iitb/os_project/workflows/Rust/badge.svg?branch=master)

The project aims to find out the effect of **preemption** and **context-switch** on **throughput** and **latency** for a large number of isolated domains(processes). The operating system overhead will increase sharply with the increase in the number of domains, especially when the total number of active domains are more than the number of cores.

The `server` forks a large number of processes(based on the configuration parameter), and `client` will randomly generate the requests for these processes.

## How to Run

1) Clone the repository.

```
git clone https://github.com/ankit-iitb/os_project
cd os_project
```

2) Install Rust

```
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustup default nightly
```
3) Build the synthetic client-server application.
```
make
```

4) Run the synthetic application with a client and server. The client sends requests
to the server, which transmits the payload of the same request in the response.

On the server:
```
cd linux/server/
./target/release/server
```

On the client:
```
cd linux/client/
./target/release/client
```

## Configuration Parameters
To update the configuration parameters, change `linux/server/server.toml` on the server side and `linux/client/client.toml` on the client side.

 Change `server_ip` and `num_process` in server.toml and `server_ip`, `num_tenants`, and `req_rate` in client.toml.
