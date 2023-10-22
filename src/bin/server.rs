use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type DB = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening...");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The second item contains the IP and port of the new connection
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();

        println!("Accepted");
        process(socket, db).await;
    }
}

async fn process(socket: TcpStream, db: DB) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            mini_redis::Command::Set(cmd) => {
                let mut db = db.lock().unwrap();
                // The value is stored as Vec<u8>
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            mini_redis::Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented!: {:?}", cmd),
        };

        // write the response to client
        connection.write_frame(&response).await.unwrap();
    }
}
