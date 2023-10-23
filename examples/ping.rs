use mini_redis::clients::Client;

#[tokio::main]
pub async fn main() -> mini_redis::Result<()> {
    let mut client = Client::connect("127.0.0.1:6379").await?;

    let res = client.ping(None).await?;

    println!("got value from the server; success={:?}", res);

    Ok(())
}
