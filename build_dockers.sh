#!/bin/bash

for variant in vaapi ubuntu scratch nvidia alpine centos; do
  local NAME=$REGISTRY/$IMAGE_NAME:$variant
  echo Building $NAME
  docker build --build-arg FFMPEG_VARIANT=3.4-$variant . -t $NAME
  docker push $NAME
done

docker tag $REGISTRY/$IMAGE_NAME:ubuntu $REGISTRY/$IMAGE_NAME:latest
docker push $REGISTRY/$IMAGE_NAME:latest