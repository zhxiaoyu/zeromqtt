# ZeroMQTT

[中文文档](README_CN.md) | English

A powerful bidirectional bridge between ZeroMQ and MQTT protocols, supporting high-performance, reliable message forwarding with a modern web dashboard.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![Status](https://img.shields.io/badge/status-beta-yellow.svg)

## Features

- **Bidirectional Bridging**: Seamless message forwarding between ZeroMQ (PUB/SUB, XPUB/XSUB) and MQTT protocols
- **Web Dashboard**: Modern Vue 3 dashboard for real-time monitoring and configuration
- **RESTful API**: Complete configuration management via HTTP API
- **Dynamic Configuration**: Hot-reload topic mappings without restarting the bridge
- **Topic Mapping**: Flexible topic transformation with MQTT wildcard support (`+`, `#`)
- **Auto Reconnection**: Built-in connection recovery for both MQTT and ZeroMQ
- **SQLite Storage**: Persistent configuration storage
- **Docker Support**: Production-ready Docker and docker-compose deployment

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        ZeroMQTT Bridge                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │ MQTT Client │───▶│ Bridge Core  │───▶│  ZMQ Publisher  │    │
│  │ (Subscriber)│    │              │    │  (PUB/XPUB)     │    │
│  └─────────────┘    │   Topic      │    └─────────────────┘    │
│                     │   Mapper     │                            │
│  ┌─────────────┐    │              │    ┌─────────────────┐    │
│  │ MQTT Client │◀───│   Stats      │◀───│  ZMQ Subscriber │    │
│  │ (Publisher) │    │   Collector  │    │  (SUB/XSUB)     │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Web Dashboard (Vue 3 + Vite)               │   │
│  │  • Real-time Statistics    • Configuration Management  │   │
│  │  • Connection Status       • Topic Mapping Editor       │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+ 
- Node.js 18+ (for dashboard development)
- An MQTT broker (or use public broker like `broker.emqx.io`)

### Build and Run

```bash
# Clone the repository
git clone https://github.com/yourusername/zeromqtt.git
cd zeromqtt

# Build the project
cargo build --release

# Run the bridge
cargo run
```

The bridge will start with:
- **Web Dashboard**: http://localhost:3000
- **API Endpoint**: http://localhost:3000/api
- **ZMQ PUB Socket**: tcp://*:5555
- **ZMQ SUB Socket**: tcp://localhost:5556

### Docker Deployment

```bash
# Using docker-compose
docker-compose up -d

# Or build manually
docker build -t zeromqtt .
docker run -p 3000:3000 -p 5555:5555 zeromqtt
```

## Configuration

### Default Endpoints

| Type | Socket | Endpoint |
|------|--------|----------|
| MQTT | - | broker.emqx.io:1883 |
| ZMQ PUB | Bind | tcp://*:5555 |
| ZMQ SUB | Connect | tcp://localhost:5556 |

### Topic Mapping

Configure mappings via the web dashboard or API:

```json
{
  "source_endpoint_type": "mqtt",
  "source_endpoint_id": 1,
  "target_endpoint_type": "zmq",
  "target_endpoint_id": 3,
  "source_topic": "sensors/+/temperature",
  "target_topic": "zmq/sensors/temperature",
  "direction": "mqtt_to_zmq",
  "enabled": true
}
```

### Wildcard Support

| Pattern | Description | Example |
|---------|-------------|---------|
| `+` | Single-level wildcard | `sensors/+/temp` matches `sensors/room1/temp` |
| `#` | Multi-level wildcard | `sensors/#` matches `sensors/a/b/c` |

## API Reference

### Status

```bash
# Get bridge status
curl http://localhost:3000/api/status

# Get message statistics
curl http://localhost:3000/api/status/stats
```

### Configuration

```bash
# List MQTT brokers
curl http://localhost:3000/api/config/mqtt

# List ZMQ endpoints
curl http://localhost:3000/api/config/zmq

# List topic mappings
curl http://localhost:3000/api/config/mappings

# Add new mapping
curl -X POST http://localhost:3000/api/config/mappings \
  -H "Content-Type: application/json" \
  -d '{"source_endpoint_type":"mqtt","source_endpoint_id":1,...}'
```

### Bridge Control

```bash
# Start bridge
curl -X POST http://localhost:3000/api/bridge/start

# Stop bridge
curl -X POST http://localhost:3000/api/bridge/stop

# Restart bridge
curl -X POST http://localhost:3000/api/bridge/restart
```

## Testing

### Unit Tests

```bash
cargo test
```

### End-to-End Tests

The E2E test suite verifies actual message flow:

```bash
# Terminal 1: Start the bridge
cargo run

# Terminal 2: Run E2E tests
cargo run --bin e2e_tests
```

E2E tests cover:
- MQTT → ZMQ message bridging
- ZMQ → MQTT message bridging
- Bidirectional bridging
- Hot reload/dynamic configuration
- Topic transformation
- Enable/disable mappings
- Multiple simultaneous mappings

## ZMQ Patterns Supported

| Pattern | Description | Use Case |
|---------|-------------|----------|
| PUB/SUB | Publish-Subscribe | Standard bridging |
| XPUB/XSUB | Extended PUB/SUB | Proxy scenarios with subscription forwarding |

## Project Structure

```
zeromqtt/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── api/             # REST API handlers
│   ├── bridge/          # Core bridging logic
│   │   ├── core.rs      # Bridge orchestration
│   │   ├── worker.rs    # MQTT/ZMQ workers
│   │   └── mapper.rs    # Topic mapping
│   ├── db/              # Database layer
│   ├── models/          # Data structures
│   └── telemetry/       # Logging & metrics
├── dashboard/           # Vue 3 web dashboard
├── tests/               # Integration & E2E tests
├── Dockerfile
└── docker-compose.yml
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [paho-mqtt](https://github.com/eclipse/paho.mqtt.rust) - MQTT client
- [zmq](https://github.com/erickt/rust-zmq) - ZeroMQ bindings
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Vue.js](https://vuejs.org/) - Dashboard framework