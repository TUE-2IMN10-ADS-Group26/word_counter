# word_counter
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](./LICENSE)

Repositories of ADS lab assignment.

## Usage
To start the Docker containers, run the following command:
```bash
docker-compose up --build
```
To view the running containers, use:
```bash
docker ps
```
Next, select the client container and attach to it with:
```bash
docker exec -it <client_container_id> /bin/bash
```
Once you're inside the client container, navigate to the executable file directory:
```bash
cd /app/target/release/
```
You can then use the client executable:
```bash
./counter_client --help
```
If everything is set up correctly, you should be able to view the metrics data in the predefined [grafana dashboard](http://localhost:3000).