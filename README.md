[![progress-banner](https://backend.codecrafters.io/progress/redis/f196d1a1-22ec-4233-b95d-74a6a087cbc6)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

In this code challenge, over **72 hours** I will build a [Redis](https://redis.io/) clone in the Rust programming language.
The best way to learn a new language, IMHO, is through transfer learning.

My hypothesis stems from the machine learning concept called "Domain Adaptation" where generalizability (learning) in one domain (i.e. building Redis in Python) can accelerate convergence to generalizability in another domain (i.e. building Redis in Rust).

By the end of this challenge, my goal is to have a Redis server clone that can stand in for the `redis-stack-server` instance and receive commands like `PING`, `ECHO`, `SET`, and `GET`.

### How to use this repo

If you would like to expand on what I've built feel free to clone this repository.

To start the Redis Server, you'll need execution privileges on the `./start_redis_server.sh` shell script.
Open two (or more) terminals, in one, build and run the **Redis Server** through the provided shell script.

```console
$ ./start_redis_server.sh
```

In the other terminal(s), you can send client commands with the `redis-cli` as follows:

```
$ redis-cli PING
+PONG
$ redis-cli SET foo bar
"Ok"
$ redis-cli GET foo
"bar"
```

# First stage - TCP Connections
Here's my gameplan for getting my **Redis Server Clone** up-and-running:

- [x] Establish a TCP server connection to the Redis port at `127.0.0.0.1:6379`
- [x] Read from the TCP connection stream via the `redis-cli` client
- [x] Write the read result to the TCP connection stream (echo the client)
- [x] Build handler and create main event loop to be able to accept multiple TCP connections

# Second stage - RESP Parser
Once the **Redis Server** can accept multiple connections, the next step will be to translate the [RESP](https://redis.io/docs/latest/develop/reference/protocol-spec/) received from the `redis-cli`:

- [x] Read input from TCP stream as buffer (i.e. `[u8]`)
- [x] Build enum for possible data types received from buffer
- [x] Filter data type handling based on first character in buffer:
   - [x] `+` = **simple text strings** ("+OK\r\n" = `RespValue::Text("OK")`)
   - [x] `$` = **bulk strings** ("$2\r\nOK\r\n" = `RespValue::BulkString("OK")`)
   - [x] `:` = **integers** (":-100\r\n" = `RespValue::Integer(-100)`)
   - [x] `*` = **arrays** ("*2\r\n$4\r\nECHO\r\n$9\r\nI am evan\r\n" = `Array(RespValue::BulkString("ECHO"), RespValue::BulkString("I am evan"))`)
- [x] Map parsing result into a command to execute (i.e. "*2\r\n$4\r\nECHO\r\n$9\r\nI am evan\r\n" = `ECHO` function with 1 string argument)
- [x] Create the basic `PING` command logic (i.e. received "+PING\r\n", hardcoded output "+PONG\r\n")
- [x] Create the parameterized `ECHO` command logic (i.e. received "*2\r\n$4\r\nECHO\r\n$2\r\nhi\r\n", hardcoded output "$2\r\nhi\r\n")

# Third stage - Basic data interactions
Now that the client can send commands to our **Redis Server**, we need to handle them. For this we'll use a basic [HashMap](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) from the `std::collections` crate for storage (`SET`) and retrieval (`GET`).

- [x] Adapt main event loop to spawn a new [Tokio future promise](https://tokio.rs/) for each connection
- [x] Create a new top-level HashMap to store all entries from `SET` commands and create a clone for each Tokio future
- [x] Add support for the `SET` command, passing a pointer to the HashMap clone for key-value insertion based on "key" and "value" arguments
- [x] Add support for the `GET` command, retrieving value or returning a null string value (`RespValue::Null` = "$-1\r\n) based on "key" argument

# Fourth stage - Upgrading data storage
Before this **Redis Server Clone** can be used as a full in-memory cache, I'll need an extensible database structure that can maintain database state and handle key-value expirations.

- [x] Add a new `Db` struct for storing the HashMap and a BTreeSet for key expirations
- [x] Store `expires_at` value in addition to the original String data for each entry in the new HashMap
- [x] Add an optional `expiry` argument to the `SET` command, converting it to the `expires_at` attribute for the new entry
- [x] Write `expires_at` to the BTreeSet for key expirations (overwrite old expiration if exists)
- [x] Spawn a background Tokio future to purge keys based on expirations BTreeSet
- [x] Set timer to keep background future asleep until next expiration time
- [x] Notify background future to awake and update its next expiration everytime client runs `SET` or `GET`