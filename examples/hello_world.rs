use std::time::Duration;

use mini_redis::{clients::Client, Result};

async fn say_world() {
    println!("world");
}

#[tokio::main]
async fn main() -> Result<()> {

    let op = say_world();

    op.await;

    println!("hello");


    // Open a connection to the mini-redis address.

    // for i in 0..10 {
    //     tokio::spawn(async move {
    //         let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    //         // Set the key "hello" with value "world"
    //         client
    //             .set("hello", format!("world {}", i).into())
    //             .await
    //             .unwrap();
    //     });
    // }

    // for _ in 0..10 {
    //     tokio::spawn(async {
    //         let mut client2 = Client::connect("127.0.0.1:6379").await.unwrap();
    //         // Get key "hello"
    //         let result = client2.get("hello").await.unwrap();

    //         println!("got value from the server; result = {:?}", result);
    //     });
    // }

    // tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
