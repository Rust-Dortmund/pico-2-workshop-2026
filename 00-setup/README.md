# Getting Started with the Pi Pico 2

In this first exercise, you will set up everything you need to have your own code run on the Raspberry Pi Pico 2W, including installing the required tools for compiling and flashing (if you haven't done so already), connecting the Pico 2 to your laptop, and running the provided "hello world" program.
This mostly serves as a checkpoint to make sure that you're ready to go and there isn't anything wrong with your setup or with the hardware that you were provided with - you'll write your own code starting from the next exercise.

If you want to look into the Pico 2 some more, you can find Raspberry's page for it [here](https://www.raspberrypi.com/products/raspberry-pi-pico-2/) or you can always have a look into the [Pico 2's datasheet](https://pip-assets.raspberrypi.com/categories/1088-raspberry-pi-pico-2-w/documents/RP-008304-DS-1-pico-2-w-datasheet.pdf?disposition=inline) or the [datasheet of the Pico 2's RP2350 processor](https://pip-assets.raspberrypi.com/categories/1214-rp2350/documents/RP-008373-DS-2-rp2350-datasheet.pdf?disposition=inline), which is very comprehensive and contains some useful illustrations.

## Installing Rust

In case this workshop is your first interaction with Rust, welcome! Great to have you here!
You can install Rust by following the instructions [here](https://www.rust-lang.org/learn/get-started) to get `rustup`, Rust's native tool for managing installations of the Rust toolchain, running

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

on Unix systems or downloading a [standalone installer](https://forge.rust-lang.org/infra/other-installation-methods.html).

Even if you've used Rust before, you'll probably need to install some additional components if you haven't worked with a Pi Pico before, such as a version of the standard library that was built for the RP2350, which you can do by running the following command after `rustup` is installed:

```shell
rustup target add thumbv8m.main-none-eabihf
```

Depending on your OS, you may or may not need to install some system packages for build dependencies as well.
For (Debian-based) Linux systems, the following should get you everything you need:

```shell
apt install -y pkg-config libudev-dev cmake git
```

## Installing `probe-rs`

We will use a tool called `probe-rs` to connect to the Pico 2 and flash it with our code.
You can download the latest release [from GitHub](https://github.com/probe-rs/probe-rs/releases/tag/v0.30.0) or build it from source using

```shell
cargo install probe-rs-tools@0.30 --locked
```

(Note that this will install the latest version at the time of preparing the workshop. If you are reading this far into the future, use a newer version at your own risk.)

Linux users will additionally need to follow the [probe setup instructions](https://probe.rs/docs/getting-started/probe-setup/) to download and install the `69-probe-rs.rules` rules for `udev`:

```sh
cp 69-probe-rs.rules /etc/udev/rules.d/
udevadm control --reload
```

If you want your terminal to autocomplete `probe-rs` commands, you can run

```shell
probe-rs complete install
```

and follow its instructions to enable shell completions.

## Connecting the Debug Probe

To use `probe-rs` and run our program, you will need to connect your laptop to the Pico 2 via the provided Raspberry Pi debug probe.
Connect the top side of the probe that only has a single micro-USB port to your laptop via the included USB cable (if your laptop doesn't have a USB-A port, use the adapter you brought).
On the bottom side, find the included cable that has a plug matching the bottom ports on _both ends_ and plug one end into the **D** port.
Then, plug the other end of the cable into the matching socket labeled "DEBUG" on the Pico 2, directly below the RP2350 processor with the printed-on Raspberry logo on the right-hand side of the PCB.
Once done, running `probe-rs list` should show that a single debug probe (labeled `[0]`) was found which is called something like `CMSIS-DAP`.

**A note for WSL users:** It is absolutely possible to use the same setup in WSL2.
However, you will need to pipe the debug probe's USB through from Windows - have a look at [this article](https://learn.microsoft.com/en-us/windows/wsl/connect-usb) that explains how to use `usbipd` to achieve this.

## Flashing and Running the Code

If you've reached this point having successfully completed the previous steps, you should now be ready to go to run your first program on the Pico 2.
Before you do, make sure to provide power to the Pico 2 by connecting its micro-USB port (located on the top end of the board) to a power source (laptop or USB charger).

The project is already configured to use `probe-rs`, so a simple `cargo run --release` from within this folder should do the trick (you should always build with `--release` for this workshop since the resulting binary will be a lot smaller) - once the Rust compiler is done building the example code, you should see two progress bars fill up while the program is being transferred onto the chip through the debug probe.

Upon success, the LED in the top-left corner of the Pico 2 will start blinking on and off.
You should also see some log messages appear in your terminal periodically (you don't need to understand any of them except for the ones that say "led on" and "led off" - they won't be relevant for the upcoming exercises).
