#![allow(clippy::manual_async_fn)]
#![allow(clippy::large_enum_variant)]

#[macro_use]
extern crate std;

use async_std::prelude::FutureExt;
use async_std::sync::Arc;
use async_std::task;
use futures::prelude::*;
use std::convert::TryFrom;
use std::str;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;
use zenoh::*;
use zrpc::ZServe;
//importing the macros
use zrpc_macros::{zserver, zservice};

use log::trace;

#[zservice(timeout_s = 10, prefix = "/lfos")]
pub trait Hello {
    async fn hello(&self, name: String) -> String;
}

#[derive(Clone)]
struct HelloZService(String);

#[zserver(uuid = "10000000-0000-0000-0000-000000000001")]
impl Hello for HelloZService {
    async fn hello(&self, name: String) -> String {
        format!("Hello {}!, you are connected to {}", name, self.0)
    }
}

#[async_std::main]
async fn main() {
    let zenoh = Arc::new(
        Zenoh::new(Properties::from("mode=client;peer=tcp/127.0.0.1:7447").into())
            .await
            .unwrap(),
    );
    // let ws = Arc::new(zenoh.workspace(None).await.unwrap());

    let service = HelloZService("test service".to_string());

    let z = zenoh.clone();
    let ser_uuid = service.instance_uuid();
    let server = service.get_hello_server(z);
    let client = HelloClient::new(zenoh.clone(), ser_uuid);

    server.connect();
    server.initialize();
    server.register();

    let local_servers = HelloClient::find_local_servers(zenoh.clone()).await;
    println!("local_servers: {:?}", local_servers);

    let servers = HelloClient::find_servers(zenoh.clone()).await;
    println!("servers found: {:?}", servers);

    // this should return an error as the server is not ready
    let hello = client.hello("client".to_string()).await;
    println!("Res is: {:?}", hello);

    let (s, handle) = server.start();

    let local_servers = HelloClient::find_local_servers(zenoh.clone()).await;
    println!("local_servers: {:?}", local_servers);

    let servers = HelloClient::find_servers(zenoh.clone()).await;
    println!("servers found: {:?}", servers);

    task::sleep(Duration::from_secs(1)).await;
    let hello = client.hello("client".to_string()).await;
    println!("Res is: {:?}", hello);

    let hello = client.hello("client_two".to_string()).await;
    println!("Res is: {:?}", hello);

    server.stop(s);

    let local_servers = HelloClient::find_local_servers(zenoh.clone()).await;
    println!("local_servers: {:?}", local_servers);

    let servers = HelloClient::find_servers(zenoh.clone()).await;
    println!("servers found: {:?}", servers);

    server.unregister();
    server.disconnect();

    handle.await;

    // this should return an error as the server is not there
    let hello = client.hello("client".to_string()).await;
    println!("Res is: {:?}", hello);
}