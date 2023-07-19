<div align="center">
 <h1>Auto Clicker for Desktops</h1>
 <p>
  <b>A portable auto clicker built for Linux, macOS & Windows.</b>
 </p>
 <br>
</div>

## Disclaimer

This app is for **educational purposes only.**

## About

This is an autoclicker app for desktop operating systems (Windows, macOS, and Linux) that allows you to automate mouse clicks. It can be useful for repetitive tasks that involve clicking the same button or area repeatedly, such as gaming or testing.

## OS specific requirements

### Fedora Rawhide (not tested)

```shell
dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel
```

### Linux

```shell
sudo apt-get install libx11-dev libxtst-dev libevdev-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev
# Install `libfontconfig-dev` if you get the following error
# error: failed to run custom build command for `servo-fontconfig-sys v5.1.0`
sudo apt-get install libfontconfig-dev
```

## Running

```shell
rustup update
cargo run --release
```

## Build

First you must install cargo bundle using cargo. To install `cargo bundle`, run `cargo install cargo-bundle`. This will add the most recent version of `cargo-bundle` published to crates.io as a subcommand to your default cargo installation.

Then use the following command to create an application bundle for your operationg system.

```shell
rustup update
cargo bundle --release
```

## License

This app is licensed under the **MIT License**, which means that you can use, modify, and distribute the code as long as you include the original license notice in any copies or modifications.
