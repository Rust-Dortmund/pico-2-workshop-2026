# Brainstorming Rust Dortmund Meetup Embedded Rust Workshop

## Prerequisites

- Install [Rust](https://www.rust-lang.org/learn/get-started)
- Install required targets and [`probe-rs`](https://probe.rs/)

  ```
  > rustup target add riscv32imc-unknown-none-elf
  > sudo apt install -y pkg-config libudev-dev cmake git
  > cargo install probe-rs-tools --locked
  > probe-rs complete install
  ```

- Do the probe setup: https://probe.rs/docs/getting-started/probe-setup/

  Basically: Download `69-probe-rs.rules` and run:

  ```
  > sudo cp 69-probe-rs.rules /etc/udev/rules.d/
  > sudo udevadm control --reload
  ```

- For ESP32C3 projects add the user to `dialout` and install `espflash`:

  ```
  > sudo usermod -a -G dialout <your user>
  > cargo install espflash --locked
  ```

  Add udev rules for on-board JTAG:

  ```
  # ESP USB JTAG Serial
  ATTRS{idVendor}=="303a", ATTRS{idProduct}=="1001", MODE="0666", ENV{ID_MM_DEVICE_IGNORE}="1", ENV{ID_MM_PORT_IGNORE}="1"
  ```

  (see https://gist.github.com/a-gavin/1923ca5fdb633150303cb2fe00571d40)

- And then REBOOT!
- If you want to use the utility build script install Python 3

## Building

Refer to the `do.py` script.
