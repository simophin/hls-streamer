FROM rust

RUN mkdir /build
WORKDIR /build
COPY . ./
RUN cargo build --release


FROM jrottenberg/ffmpeg:3-vaapi

RUN mkdir /app
WORKDIR /app
COPY --from=0 /build/target/release/hls-streamer ./

ENV FFMPEG_INPUT=""
ENV HLS_DIR="/data"

ENTRYPOINT /app/hls-streamer