<h1 align="center">mplayer-server</h1>

A [DBus](https://www.freedesktop.org/wiki/Software/dbus/#whatisd-bus) music player server wirtten in [Rust](https://www.rust-lang.org)

# Thanks to
- [Zbus](https://github.com/dbus2/zbus)
- [Rodio](https://github.com/RustAudio/rodio.git)
- [Tokio](https://github.com/tokio-rs/tokio)
- [Serde](https://github.com/serde-rs/serde)
- [Loft-rs](https://github.com/Serial-ATA/lofty-rs)
> and many more...

# Usage:

## Installation
clone the repo
```bash
git clone --depth=1 https://github.com/yassinebenarbia/mplayer-server.git
```
then install it using [cargo](https://doc.rust-lang.org/cargo/)
```bash
cargo install --features "debug" --path ./mplayer-server/
```
and to run it you just run this, make sure to have the configuration set-up before, **passe the config file path as an argument or put it on `$HOME/.config/mplayer-server/config.toml`**, check [configuration](#Configuration) for details
```
mplayer-server # optional path here
```
>[!NOTE]
> If you want to maintain this package on any distro feel free to do so, you can contact me or leave an Issue if you any question

## Configuration
Check a the configuration on [this example config file](./config.example.toml), for now, you have to put the file on `$HOME/.config/mplayer-server/config.toml` or provide the path of the config as the first commmand line parameter.

## Documentation
access documentation on [DOCS.md](./DOCS.md)

# Feature Flags
|Feature  |Description                                                                                  |
|---------|---------------------------------------------------------------------------------------------|
|debug    |Enables display debug information/logs (you probebly want to compile it with this)           |
|full-log |Shows more depth logs (require `debug` feature to work)                                      | 
|trivial  |Shows trival log informations, more depth thatn `full-log`. (require `debug` feature to work)|
