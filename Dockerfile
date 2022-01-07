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
ENV LISTEN_ADDRESS="0.0.0.0"
ENV LISTEN_PORT=8989
ENV TIMEOUT=120

EXPOSE 8989

ENTRYPOINT /app/hls-streamer