## Usage

```shell
./target/debug/mqtt-bench --help
A MQTT benchmark tool

Usage: mqtt-bench [COMMAND]

Commands:
  connect    
  pub        
  sub        
  benchmark  
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### CONNECT

```shell
./target/debug/mqtt-bench connect --help
Usage: mqtt-bench connect [OPTIONS] --host <HOST> --username <USERNAME> --password <PASSWORD>

Options:
      --host <HOST>
          
  -p, --port <PORT>
          
  -u, --username <USERNAME>
          
  -P, --password <PASSWORD>
          
  -s, --ssl
          
  -v, --verify
          
  -a, --auth-server-certificate
          
  -q, --qos <QOS>
          [default: 1]
  -n, --start-number <START_NUMBER>
          [default: 0]
      --total <TOTAL>
          Total number of client to create [default: 16]
  -c, --concurrency <CONCURRENCY>
          The number of clients to create in parallel for each iteration [default: 4]
  -i, --interval <INTERVAL>
          The interval between each message publishing for each client in milliseconds [default: 100]
      --time <TIME>
          The duration of the test in seconds [default: 60]
      --client-id <CLIENT_ID>
          [default: BenchClient%d]
      --show-statistics
          
      --connect-timeout <CONNECT_TIMEOUT>
          [default: 5]
      --keep-alive-interval <KEEP_ALIVE_INTERVAL>
          [default: 3]
      --max-inflight <MAX_INFLIGHT>
          [default: 1024]
  -h, --help
          Print help
```

#### Sample Run

```shell
RUST_LOG=info cargo run -- connect --host localhost --username user0 --password secret0 --total 16 -c 2 --time 10
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.53s
     Running `target/debug/mqtt-bench connect --host localhost --username user0 --password secret0 --total 16 -c 2 --time 10`
[2024-12-03T02:02:27.811Z INFO  mqtt_bench::state] Client Summary[Attempted:2, Connected: 2, Disconnected: 14] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:28.814Z INFO  mqtt_bench::state] Client Summary[Attempted:2, Connected: 2, Disconnected: 14] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:29.412Z INFO  mqtt_bench::command] All clients have connected and it is time to count down running time.
[2024-12-03T02:02:29.816Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:30.818Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:31.821Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:32.825Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:33.828Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:34.830Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:35.833Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:36.836Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:37.840Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:38.842Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 0, Failure: 0], Subscribed: 0
[2024-12-03T02:02:39.435Z INFO  mqtt_bench::statistics] Connect Latency Histogram P90: 800ms, P95: 800ms, P99: 800ms
```

### PUBLISH

```shell
./target/debug/mqtt-bench pub --help
Usage: mqtt-bench pub [OPTIONS] --host <HOST> --username <USERNAME> --password <PASSWORD>

Options:
      --host <HOST>
          

  -p, --port <PORT>
          

  -u, --username <USERNAME>
          

  -P, --password <PASSWORD>
          

  -s, --ssl
          

  -v, --verify
          

  -a, --auth-server-certificate
          

  -q, --qos <QOS>
          [default: 1]

  -n, --start-number <START_NUMBER>
          [default: 0]

      --total <TOTAL>
          Total number of client to create
          
          [default: 16]

  -c, --concurrency <CONCURRENCY>
          The number of clients to create in parallel for each iteration
          
          [default: 4]

  -i, --interval <INTERVAL>
          The interval between each message publishing for each client in milliseconds
          
          [default: 100]

      --time <TIME>
          The duration of the test in seconds
          
          [default: 60]

      --client-id <CLIENT_ID>
          [default: BenchClient%d]

      --show-statistics
          

      --connect-timeout <CONNECT_TIMEOUT>
          [default: 5]

      --keep-alive-interval <KEEP_ALIVE_INTERVAL>
          [default: 3]

      --max-inflight <MAX_INFLIGHT>
          [default: 1024]

      --topic <TOPIC>
          Topic pattern to publish messages to.
          
          The topic pattern can contain a `%d` placeholder which will be replaced by an ID.
          
          For example, if the topic pattern is `home/%d`, the actual topic will be `home/0`, `home/1`, etc.
          
          [default: home/%d]

      --topic-total <TOPIC_TOTAL>
          If `topic` contains `%i`, this is the number of topics to publish messages to.
          
          If `topic_total` is less than number of the clients: `total`, the topics will be reused; If the `topic_total` is greater than the number of clients, only the first `total` topics will be used during benchmark;
          
          If `topic_total` is 0, it will be set to `total`.
          
          [default: 0]

      --message-size <MESSAGE_SIZE>
          [default: 64]

      --payload <PAYLOAD>
          

  -h, --help
          Print help (see a summary with '-h')
```

### SUBSCRIBE

```shell
./target/debug/mqtt-bench sub --help
Usage: mqtt-bench sub [OPTIONS] --host <HOST> --username <USERNAME> --password <PASSWORD> --topic <TOPIC>

Options:
      --host <HOST>
          

  -p, --port <PORT>
          

  -u, --username <USERNAME>
          

  -P, --password <PASSWORD>
          

  -s, --ssl
          

  -v, --verify
          

  -a, --auth-server-certificate
          

  -q, --qos <QOS>
          [default: 1]

  -n, --start-number <START_NUMBER>
          [default: 0]

      --total <TOTAL>
          Total number of client to create
          
          [default: 16]

  -c, --concurrency <CONCURRENCY>
          The number of clients to create in parallel for each iteration
          
          [default: 4]

  -i, --interval <INTERVAL>
          The interval between each message publishing for each client in milliseconds
          
          [default: 100]

      --time <TIME>
          The duration of the test in seconds
          
          [default: 60]

      --client-id <CLIENT_ID>
          [default: BenchClient%d]

      --show-statistics
          

      --connect-timeout <CONNECT_TIMEOUT>
          [default: 5]

      --keep-alive-interval <KEEP_ALIVE_INTERVAL>
          [default: 3]

      --max-inflight <MAX_INFLIGHT>
          [default: 1024]

      --topic <TOPIC>
          

      --topic-total <TOPIC_TOTAL>
          If `topic` contains `%i`, this is the number of topics to publish messages to.
          
          If `topic_total` is less than number of the clients: `total`, the topics will be reused; If the `topic_total` is greater than the number of clients, only the first `total` topics will be used during the benchmark.
          
          If `topic_total` is 0, it will be set to `total`.
          
          [default: 0]

  -h, --help
          Print help (see a summary with '-h')
```

### Benchmark

```shell
./target/debug/mqtt-bench benchmark --help
Usage: mqtt-bench benchmark [OPTIONS] --host <HOST> --username <USERNAME> --password <PASSWORD>

Options:
      --host <HOST>
          

  -p, --port <PORT>
          

  -u, --username <USERNAME>
          

  -P, --password <PASSWORD>
          

  -s, --ssl
          

  -v, --verify
          

  -a, --auth-server-certificate
          

  -q, --qos <QOS>
          [default: 1]

  -n, --start-number <START_NUMBER>
          [default: 0]

      --total <TOTAL>
          Total number of client to create
          
          [default: 16]

  -c, --concurrency <CONCURRENCY>
          The number of clients to create in parallel for each iteration
          
          [default: 4]

  -i, --interval <INTERVAL>
          The interval between each message publishing for each client in milliseconds
          
          [default: 100]

      --time <TIME>
          The duration of the test in seconds
          
          [default: 60]

      --client-id <CLIENT_ID>
          [default: BenchClient%d]

      --show-statistics
          

      --connect-timeout <CONNECT_TIMEOUT>
          [default: 5]

      --keep-alive-interval <KEEP_ALIVE_INTERVAL>
          [default: 3]

      --max-inflight <MAX_INFLIGHT>
          [default: 1024]

      --topic <TOPIC>
          Topic pattern to publish messages to.
          
          The topic pattern can contain a `%d` placeholder which will be replaced by an ID.
          
          For example, if the topic pattern is `home/%d`, the actual topic will be `home/0`, `home/1`, etc.
          
          [default: home/%d]

      --topic-total <TOPIC_TOTAL>
          If `topic` contains `%i`, this is the number of topics to publish messages to.
          
          If `topic_total` is less than number of the clients: `total`, the topics will be reused; If the `topic_total` is greater than the number of clients, only the first `total` topics will be used during benchmark;
          
          If `topic_total` is 0, it will be set to `total`.
          
          [default: 0]

      --message-size <MESSAGE_SIZE>
          [default: 64]

      --payload <PAYLOAD>
          

  -h, --help
          Print help (see a summary with '-h')
```

#### Sample Run
```shell
RUST_LOG=info cargo run -- benchmark --host localhost --username user0 --password secret0 --total 16 -c 2 --time 10 --topic home
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.69s
     Running `target/debug/mqtt-bench benchmark --host localhost --username user0 --password secret0 --total 16 -c 2 --time 10 --topic home`
[2024-12-03T02:07:42.715Z INFO  mqtt_bench] Now that --topic-total is 0, it will be set to --topic-total=16
[2024-12-03T02:07:43.716Z INFO  mqtt_bench::state] Client Summary[Attempted:8, Connected: 8, Disconnected: 8] Publish: [Success: 35, Failure: 0], Subscribed: 173
[2024-12-03T02:07:44.718Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 139, Failure: 0], Subscribed: 1928
[2024-12-03T02:07:45.321Z INFO  mqtt_bench::command] All clients have connected and it is time to count down running time.
[2024-12-03T02:07:45.719Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 160, Failure: 0], Subscribed: 2560
[2024-12-03T02:07:46.722Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 144, Failure: 0], Subscribed: 2304
[2024-12-03T02:07:47.724Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 160, Failure: 0], Subscribed: 2560
[2024-12-03T02:07:48.726Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 144, Failure: 0], Subscribed: 2304
[2024-12-03T02:07:49.729Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 148, Failure: 0], Subscribed: 2368
[2024-12-03T02:07:50.730Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 151, Failure: 0], Subscribed: 2304
[2024-12-03T02:07:51.733Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 153, Failure: 0], Subscribed: 2560
[2024-12-03T02:07:52.735Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 157, Failure: 0], Subscribed: 2358
[2024-12-03T02:07:53.737Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 147, Failure: 0], Subscribed: 2506
[2024-12-03T02:07:54.739Z INFO  mqtt_bench::state] Client Summary[Attempted:16, Connected: 16, Disconnected: 0] Publish: [Success: 146, Failure: 0], Subscribed: 2336
[2024-12-03T02:07:55.338Z INFO  mqtt_bench::statistics] Connect Latency Histogram P90: 100ms, P95: 100ms, P99: 100ms
[2024-12-03T02:07:55.338Z INFO  mqtt_bench::statistics] Publish MQTT Message Latency P90: 10ms, P95: 20ms, P99: 30ms
[2024-12-03T02:07:55.338Z INFO  mqtt_bench::statistics] E2E MQTT Message Delivery Latency P90: 20ms, P95: 20ms, P99: 30ms
```

## Logging
To troubleshoot, we may adjust level of logging by module. For example, if we wish to diagnose underlying MQTT interaction,
we may use the following environment variable
```shell
RUST_LOG="paho_mqtt=info,paho_mqtt_c=debug"
```
