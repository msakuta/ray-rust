[![Rust](https://github.com/msakuta/ray-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/msakuta/ray-rust/actions/workflows/rust.yml)

# Ray-rust

A very simple Rust implementation of ray tracing renderer.

## Output example

![img](images/example.png)

## Usage

    USAGE:
        ray-rust.exe [FLAGS] [OPTIONS] <width> <height>

    FLAGS:
        -h, --help         Prints help information
        -m, --raymarch     Use ray marching
        -V, --version      Prints version information
        -w, --webserver    Launch a web server that responds with rendered image, rather than producing static images.
                           Good for interactive session.

    OPTIONS:
        -d, --deserialize_file <deserialize_file>
                File name for deserialized scene input. If omitted, default scene is loaded.

        -g, --gloweffect <gloweffect>
                Enable glow effect and set its strength when ray marching method is used

        -o, --output <output>                        Output file name [default: foo.png]
        -p, --port_no <port_no>                      Port number, if use web server [default: 3000]
        -s, --serialize_file <serialize_file>        File name for serialized scene output. If omitted, scene is not output.
        -t, --threads <threads>                      thread count [default: 8]

    ARGS:
        <width>     Width of the image [px]
        <height>    Height of the image [px]


## Note on web server

Web server is now feature gated, so you need to enable when you build the application, e.g.

    cargo b --features webserver
