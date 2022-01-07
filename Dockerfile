FROM rust

RUN mkdir /build
WORKDIR /build
COPY . ./
RUN cargo build --release


ARG FFMPEG_TAG=3-vaapi
FROM jrottenberg/ffmpeg:${FFMPEG_TAG}

RUN mkdir /app
WORKDIR /app
COPY --from=0 /build/target/release/hls-streamer ./

ENV FFMPEG_INPUT=""
ENV HLS_DIR="/data"

ENTRYPOINT /app/hls-streamer