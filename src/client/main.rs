use crustyring::error::Result;

use crustyring::dht::service::DhtNodeService;

use crustyring::rpc::dht::{OperationType, Query};
use rand::Rng;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut registry_client = DhtNodeService::try_connect_registry().await?;

    println!("Enter DHT query (Get, Set or Delete).\n  Type exit to quit.");

    loop {
        print!("> ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let words = input.trim().split(' ').collect::<Vec<&str>>();
        let operation = words[0].to_uppercase();

        let nodes = registry_client
            .get_connected_nodes(Request::new(()))
            .await?;
        let nodes = &nodes.get_ref().nodes;
        let node = &nodes[rand::thread_rng().gen_range(0..nodes.len())];
        let mut dht = DhtNodeService::try_connect_node(node).await?;

        match &operation[..] {
            "SET" => {
                if words.len() < 3 {
                    println!("You must provide key and value for SET query.");
                    continue
                }
                let key = words[1].to_string();
                let value = words[2].to_string();
                let request = Request::new(Query {
                    ty: OperationType::Set.into(),
                    key: key.as_bytes().to_vec(),
                    value: Some(value.as_bytes().to_vec()),
                });
                let result = dht.query_dht(request).await?;
                match &result.get_ref().error {
                    Some(err) => {
                        println!("Error: {}", err);
                    }
                    None => match result.get_ref().value.as_ref() {
                        Some(prev) => println!(
                            "Previous value was: {}, inserting: {}",
                            String::from_utf8(prev.clone()).unwrap(),
                            value
                        ),
                        None => println!("Inserting new pair ({}, {})", key, value),
                    },
                }
            }
            "DELETE" => {
                if words.len() < 2 {
                    println!("You must provide a key for DELETE query.");
                    continue
                }
                let key = words[1].to_string();
                let request = Request::new(Query {
                    ty: OperationType::Delete.into(),
                    key: key.as_bytes().to_vec(),
                    value: None,
                });
                let result = dht.query_dht(request).await?;
                match &result.get_ref().error {
                    Some(err) => {
                        println!("Error: {}", err);
                    }
                    None => match result.get_ref().value.as_ref() {
                        Some(prev) => {
                            println!(
                                "Deleting: ({}, {})",
                                key,
                                String::from_utf8(prev.clone()).unwrap()
                            )
                        }
                        None => println!("Key not present"),
                    },
                }
            }
            "GET" => {
                if words.len() < 2 {
                    println!("You must provide a key for GET query.");
                    continue
                }
                let key = words[1].to_string();
                let request = Request::new(Query {
                    ty: OperationType::Get.into(),
                    key: key.as_bytes().to_vec(),
                    value: None,
                });
                let result = dht.query_dht(request).await?;
                match &result.get_ref().error {
                    Some(err) => {
                        println!("Error: {}", err);
                    }
                    None => match result.get_ref().value.as_ref() {
                        Some(val) => {
                            println!("Value is: {}", String::from_utf8(val.clone()).unwrap())
                        }
                        None => println!("Key not present"),
                    },
                }
            }
            "EXIT" => return Ok(()),
            _ => println!("invalid entry"),
        };
    }
}
