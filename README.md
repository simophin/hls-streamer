# hls-streamer
Stream your heart's content with HLS. The HLS serving process will only start when someone accesses the stream!


## Movtivation
I've got a CCTV camera from aliexpress, I know I can use [ffmpeg hls demuxer](https://ffmpeg.org/ffmpeg-formats.html#hls-2) 
to split up the stream into HLS segments and serve nginx over the files to make an HLS stream.

But then I learned that my camera can only do H265 stream, while HLS supports H265, no mainstream browsers support H265 natively.
FFMPEG actually tries to transcode the video on the fly and it is a very expensive operation, even with hardware support: 
400% software vs 100% hardware CPU usage is not good enough. I've come up with this idea that only when someone accesses
the stream should the transcoding start. 

## Highlight
* Written in rust, minimal resource usage: the app itself takes <5K memory to run (on my machine). ffmpeg runs on
separate process and take whatever it needs though.
* Streaming only starts when someone tries to access to stream. 
* Streaming stops automatically when the access stops.
* Comes with a simple http server and it's ready to use.

## Usage

### Docker

For an onvif compatible camera, we can use its rtsp stream.

```bash
$ docker run -d \
  -e FFMPEG_INPUT="-i -rtsp_transport tcp -i rtsp://NAME:PASSWORD@CAMERA_IP/onvif2"
  ghcr.io/simophin/hls-streamer
```

