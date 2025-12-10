# Brainstorming Rust Dortmund Meetup Embedded Rust Workshop

## Prerequisites

- Install [Rust](https://www.rust-lang.org/learn/get-started)
- Install required targets and [`probe-rs`](https://probe.rs/)

  ```
  > rustup target add thumbv8m.main-none-eabihf
  > sudo apt install -y pkg-config libudev-dev cmake git
  > cargo install probe-rs-tools --locked
  > probe-rs complete install
  ```

- **On Linux**: Do the probe setup: https://probe.rs/docs/getting-started/probe-setup/

  Basically: Download `69-probe-rs.rules` and run:

  ```
  > sudo cp 69-probe-rs.rules /etc/udev/rules.d/
  > sudo udevadm control --reload
  ```

- And then REBOOT!
