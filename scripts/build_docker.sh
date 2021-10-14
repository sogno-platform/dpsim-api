!#/bin/bash

docker build -t dpsim-api .
docker tag dpsim-api:latest localhost:5000/dpsim-api
docker push localhost:5000/dpsim-api

