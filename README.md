
# Rest API for DPsim

## This is a work in progress. Very little is currently implemented.

### Read [the documentation](https://sogno-platform.github.io/dpsim-api/dpsim_api/index.html)

### Start it up

```bash
cargo run
```

This will compile and run the API Server.

### Run the tests

```bash
cargo test
```

### See it in action

```bash
docker-compose up -d rabbitmq redis-master
./curl.sh
```

The first command will start two required services, rabbitmq for aqmp messages and redis for simulation detail caching. The second command will fire a POST request at the endpoint http://localhost:8000/simulation. The file testdata/load_profile_data.zip will be used to create a simulation stub.

### Generate the documentation

```bash
cargo doc --no-deps
```


