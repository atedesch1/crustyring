# Crusty Ring

Crusty Ring is a simple Distributed Hash Table (DHT) implementation using Consistent Hashing.

- Hash ring has positions ranging from 0 to u64::MAX (2^64 - 1).
- Node's ID and DHT keys position them on the ring.
- DHT keys are uniformly distributed on the nodes in the network.
- Nodes maintain references to previous and next neighbors on the ring.
- Nodes communicate using gRPC.

![consistent-hashing](https://github.com/atedesch1/crustyring/assets/64045396/e34039d5-f7e8-474d-bb7e-deebab80f87b)

Picture shows a Consistent Hashing Ring with node's IDs and keys using a 6-digit hexadecimal number. Node C3F22A is reponsible for all keys from its ID to its next neighbor DE1A67, this includes key CD1A35 that lies in between the two nodes.


## Registry Service
Registry is a service responsible for configuring new nodes. It calculates the joining node's ID by using SHA-2 and refers it to the node that has the closest smaller ID to the node. 

## DHT Nodes
DHT nodes are responsible for storing keys in the DHT ranging from their ID to their neighbor that has the closest bigger ID (next neighbor). 

Any of them can serve requests to the DHT. If the key in the request is not present in the current node, the node forwards the request to its neighbor that is closest to the key. 

## Running the DHT

### Docker

Requirements:
- docker
- docker-compose

To build and run the registry and DHT node images run inside the root directory:
```bash
docker-compose -f clusters/docker-compose.yml up --build
```

### Local

In order to run the DHT you must first compile the project using cargo in the root of the project.
```bash
cargo build --release
```
This will build the binaries for the registry service and dht nodes.

Next, you must spin up the registry service:
```bash
./target/release/registry
```
This spawns the registry service listening on port 50000, make sure to not use this port for the DHT nodes.

Then, spin up how many DHT nodes you like by supplying a port (usually 50001, 50002, ...):
```bash
./target/release/dht <port>
```

## Querying the DHT

You can either use the provided DHT client binary to query the DHT through a simple CLI application or you can use a service such as Postman by supplying the proto/dht.proto file. We will use the DHT client:

### Docker

To build and run the client image run inside the root directory (run after the dht has initialized):
```bash
docker build -t client:latest -f clusters/Dockerfile.client .
docker run --network=dht -it client:latest
```
### Local

Spin up the DHT CLI Client:
```bash
./target/release/client
```

Either local or with docker, to query the DHT provide a Get, Set or Delete command to the CLI:
```bash
<command> <key> <value>  # Example: SET 777 abc
```
For each command given the client will pick a random DHT node in the network to make the request to and respond appropriately.


### TODO

- [x] Split up QueryDHT into QueryDHT and ForwardQuery so you can have a key of type Vec<u8> be converted to u64 and then forwarded
- [x] Transfer keys on node join
- [x] Implement simple test binary to make requests to dht
- [x] Dockerize dht nodes & make script to easily spin up everything
- [ ] Write automated tests for the dht as a whole
- [ ] Handle simultaneous node joins?
- [ ] Handle node failures by removing from registry and fixing broken connections
- [ ] Remove registry, join network by providing the address of one node in the network
- [ ] Use data replication to ensure fault tolerance
- [ ] Implement logging service to provide persistence to the dht


### Motivation

This DHT implementation is a study project that serves as the initial steps for my Bachelor thesis "A Pastry based distributed database using Rust".
