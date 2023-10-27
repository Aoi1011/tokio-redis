use mini_redis::{clients::Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let client = Client::connect("127.0.0.1:6379").await?;

    let mut subscriber = client.subscribe(vec!["foo".into()]).await?;

    if let Some(msg) = subscriber.next_message().await? {
        println!(
            "got message from the channel: {}; message = {:?}",
            msg.channel, msg.content
        );
    }

    Ok(())
}
