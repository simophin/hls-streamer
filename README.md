# hls-streamer
Stream your heart's content with HLS. 


## Movtivation
I've got a CCTV camera from AliExpress, I know I can use [ffmpeg hls demuxer](https://ffmpeg.org/ffmpeg-formats.html#hls-2) 
to split up the stream into HLS segments and serve nginx over the files to make an HLS stream.

But then I learned that my camera can only do H265 stream, while HLS supports H265, no mainstream browsers support H265 natively.
FFMPEG tries to transcode the video on the fly and it is a very expensive operation, even with hardware support: 
400% software vs 100% hardware CPU usage. I've come up with this idea that only when someone accesses
the stream should the transcoding start. 

## Highlight
* Written in Rust, minimal resource usage: the app itself takes <5K memory to run[^1].
* FFMPEG/transcoding only starts first time accessing the stream. 
* FFMPEG/transcoding stops when access stops.
* Comes with a simple web page: it's ready to view the result!

## Installation

### Docker

For an onvif compatible camera, we can use its rtsp stream.

```bash
$ docker run -d \
  -e FFMPEG_INPUT="-rtsp_transport tcp -i rtsp://NAME:PASSWORD@CAMERA_IP/onvif2"
  ghcr.io/simophin/hls-streamer
```

### Docker compose

Refer to the rest of the documentation for the usage of `FFMPEG_INPUT`.

docker-compose.yaml

```yaml
services:  
  webcam:
    image: ghcr.io/simophin/hls-streamer:latest
    restart: always
    ports:
      - 8989:8989
    devices:
      # Device mapping only needed for hardware acceleration 
      - /dev/dri/renderD128:/dev/dri/renderD128
    volumes:
      # Optional. Use a tmpfs for data storage to reduce writes to your disks.
      - hls-data:/data
    environment:
      - FFMPEG_INPUT=-vaapi_device /dev/dri/renderD128 -rtsp_transport tcp -i rtsp://USER:PASS@CAMERA_IP/onvif2 -vf format=nv12,hwupload -b:v 2M -c:v h264_vaapi

volumes:
  hls-data:
    driver: local
    driver_opts:
      type: tmpfs
      device: tmpfs
      o: size=200m
```

### Build from source

You'll need to set up Rust toolchain first, see [rustup](https://rustup.rs/)

```bash
$ cargo build --release
```

You'll also need to have ffmpeg>4.0 installed to use the app. 

Only Linux system is tested at this time.

## Configuration

### Environment varaiables

| Name            | Default value | Description                                                                                                             |
|-----------------|---------------|-------------------------------------------------------------------------------------------------------------------------|
| FFMPEG_INPUT    |               | FFMPEG input parameters.  e.g. `-rtsp_transport tcp -i rtsp://NAME:PASSWORD@CAMERA_IP/onvif2`. See below for examples.  |
| HLS_DIR         | /data         | HLS data storage directory. It's recommended to use a memory based system like tmpfs to avoid frequent write to disks.  |
| LISTEN_ADDRESS  | 0.0.0.0       | Http server listening address                                                                                           |
| LISTEN_PORT     | 8989          | Http server listening port                                                                                              |
| TIMEOUT_SECONDS | 120           | The waiting time before a stream is considered idle.                                                                    |

## Usage

Once you have the app running, by default, you will have these two links:

`http://localhost:8989` -> a simple webpage showing the HLS stream

`http://localhost:8989/master.m3u8` -> the HLS playlist itself

The playlist file request will be withheld until the playlist file is generated. This
is to avoid the first time you access the playlist because ffmpeg is not ready and you'd have
got a 404 for that file.

## Screenshots

![Screen Shot 2022-01-08 at 4 16 38 PM](https://user-images.githubusercontent.com/273191/148629726-9662c3ac-c161-4279-8eed-fc87fb6fbe38.png)


## Input examples

These examples demostrate what you can put into `FFMPEG_INPUT` environment variable.

### Copying the camera stream (i.e. no transcoding)

`-rtsp_transport tcp -i rtsp://NAME:PASSWORD@CAMERA_IP/onvif2 -vf copy`

### Hardware H264 encoding (VAAPI)

`-vaapi_device /dev/dri/renderD128 -rtsp_transport tcp -i rtsp://NAME:PASSWORD@CAMERA_IP/onvif2 -vf format=nv12,hwupload -b:v 2M -c:v h264_vaapi`


[^1]: The measurement was indicative only. FFMPEG process is excluded.
