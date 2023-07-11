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

## Running DHT
In order to run the DHT you must first compile the project using cargo in the root of the project.
```
cargo build
```
This will build the binaries for the registry service and dht nodes.

### Initializing DHT

Next, you must spin up the registry service:
```
cargo run --bin=registry
```
This spawns the registry service listening on port 50000, make sure to not use this port for the DHT nodes.

Finally, spin up how many DHT nodes you like by supplying a port (usually 50001, 50002, ...):
```
cargo run --bin=dht <port>
```
### Making Requests
Use a service that can make gRPC requests such as Postman or build your own. 

Make requests to the DHT by suppliying proto/dht.proto file to Postman and using QueryDHT request on a running node.

Example request:
```
{
    "ty": "Set",               # Get, Set, Delete
    "key": "1",                # 0 to 2^64 - 1
    "value": "Ru1nnAfPu3Ya4v"  # Optional<Any>
}
```
This will make a SET Key:Value = 1:Ru1nnAfPu3Ya4v request to the DHT.

### TODO
- [x] Split up QueryDHT into QueryDHT and ForwardQuery so you can have a key of type Vec<u8> be converted to u64 and then forwarded
- [x] Transfer keys on node join
- [] Implement simple test binary to make requests to dht
- [] Handle node failures by removing from registry and fixing broken connections
- [] Remove registry, join network by providing the address of one node in the network
- [] Dockerize dht nodes & make script to easily spin up everything
- [] Use data replication to ensure fault tolerance
- [] Implement logging service to provide persistence to the dht


### Motivation
This DHT implementation is a study project that serves as the initial steps for my Bachelor thesis "A Pastry based distributed database using Rust".
