# Loader Balancer

## Introduction

This component is designed to enhance system scalability by distributing traffic across multiple instances of
WordCounter service. It also provides additional fault tolerance features.

## Load Balancing Algorithms

The balancer supports the following algorithms:

* Round Robin
* Weighted Round Robin
* Hash (by request body)

## Configuration

All configuration files are located in the src/config directory.

### endpoints.toml

This file configures all word counter server instances.

```toml
[[endpoints]]
name = "server1" # Unique name for each server, used to distinguish them in the Grafana dashboard.
ip = "192.168.1.10" # IP address of the server, see docker-compose.yml in project root.
port = 50051 # Port on which the server listens for requests.
weight = 80 # Optional. Should be in the range (0, 100) and is only used with the Weighted Round Robin load balancing strategy.
```

### load_balancer.toml

This file contains a single field for configuration:
```toml
strategy = "RoundRobin" # Specifies the load balancing strategy. If not provided, Round Robin will be used as the default.
```

### server.toml

```bash
ip = "0.0.0.0" # IP address to bind the server to.
port = 8080 # Port for the server to listen for incoming requests.
metrics_port = 8081 # Port for exporting metrics data.
enable_fault_tolerance = true # Enables fault tolerance. Set to false to disable (phase 2).
```
