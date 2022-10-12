#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )/..
pushd $SCRIPT_DIR && echo "Changed to $SCRIPT_DIR"

if [ "x$1" == "xdev" ]
then
	docker build -f Dockerfile.dev -t dpsim:api . && \
	docker tag dpsim:api localhost:5000/dpsim:api && \
	docker push localhost:5000/dpsim:api
else
	docker build -f Dockerfile -t dpsim:api . && \
	docker tag dpsim:api localhost:5000/dpsim:api && \
	docker push localhost:5000/dpsim:api
fi

if [ "$?" != "0" ];
then
  sleep 20
  docker push localhost:5000/dpsim-api
fi
