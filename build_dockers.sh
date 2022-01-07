#!/bin/bash

docker build -t $REGISTRY/$IMAGE_NAME:latest .
docker push $REGISTRY/$IMAGE_NAME:latest