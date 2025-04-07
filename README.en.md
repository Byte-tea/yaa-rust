<h1 align="center">
  <a href="https://github.com/Byte-tea/yaa/">
    <img src="assets/yaa.svg" width="150" height="150" alt="banner" /><br>
  </a>
</h1>

<p align="center"><a href="README.md">简体中文</a> | English</p>

## yaa

`yaa` means Yet Another Agent, which can analyze and understand natural language instructions and automatically create, plan, execute, and check tasks.

## Quick Start

### Deployment

Deploy locally:

```bash
# Clone project repository to local
git clone https://github.com/Byte-tea/yaa.git
# Enter project directory
cd yaa
```

Configuration:

```bash
# Create client config file (will override default config)
cp configs/Config.example.json Config.json
# Create server config file (will override default config)
cp configs/ServerConfig.example.json ServerConfig.json
```

### Usage

Start in server mode:

```bash
cargo run --release -- --serve --port 12345
```

Start in interactive CLI mode:

```bash
cargo run --release
> Please write a bubble sort algorithm in Python.
* Working on it...
```

## Project Implementation

See [yaa documentation](docs/README.md) for details.
