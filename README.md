# Mini Redis

## Main Parts
- Runtime
- Resources (i/o, )
- utility
- 

## Runtime
multithread scheduler, 


current thread scheduler
- 

LocalSet

## Mutex
std::sync::Mutex


ReadExt



WriteExt


tokio::sync::Mutex


Parallelism
- `tokio::spawn`
- `JoinSet`


## Overview
User can check the weather through this app. If go to the link here `/weather/{city}`, then they can check.

## Command

Start the server:

```bash

RUST_LOG=debug cargo run --bin mini-redis-server

```

## Resrouces
- https://tokio.rs/blog/2019-10-scheduler


## API
- [TOKIO Doc](https://tokio.rs/tokio/)
- [TOKIO tutorial](https://tokio.rs/tokio/tutorial)
