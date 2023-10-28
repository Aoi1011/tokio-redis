use bytes::Bytes;
use mini_redis::clients::Client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        // Send the GET request
        tx.send(cmd).await.unwrap();

        // Await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_tx,
        };

        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await;

        println!("GOT = {:?}", res);
    });

    let manager = tokio::spawn(async move {
        let mut client = Client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let val = client.get(&key).await;
                    let _ = resp.send(val);
                }
                Command::Set { key, value, resp } => {
                    let val = client.set(&key, value).await;
                    let _ = resp.send(val);
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
