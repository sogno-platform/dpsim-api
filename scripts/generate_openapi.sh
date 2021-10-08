#!/bin/bash
cargo test test_get_openapi -- --nocapture 2>&1 | grep OPENAPI | sed 's/OPENAPI: //' | jq > target/doc/openapi.json
