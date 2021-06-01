
# Rest API for DPsim

## This is a work in progress. Very little is currently implemented.

### Read [the documentation](docs/dpsim_api/index.html)

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
./curl.sh
```

This will fire a POST request at the endpoint http://localhost:8000/simulation. If the file ~/example.zip should exist, it will be written to /tmp.

### Generate the documentation

```bash
cargo doc --no-deps
```


