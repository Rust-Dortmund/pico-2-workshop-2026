# Bluetooth over Ethernet

This example extends the WiFi example with Bluetooth capabilities, allowing you to also read the current LED color and change it over BLE.

TrouBLE docs: https://embassy.dev/trouble/

> [!NOTE]
> Like the WiFi example, running this example requires a nightly Rust toolchain (for picoserve's ITAIT routing).

> [!NOTE]
> `trouble-host` 0.7 is pulled from crates.io. Published `cyw43` 0.7.0 still depends on `bt-hci` 0.8, which is incompatible with `trouble-host` 0.7 (`bt-hci` 0.9). This crate therefore patches the Embassy stack from a pinned [git revision](https://github.com/embassy-rs/embassy/commit/c708723686ce80f83c80b059f04446c732db7272) (see `[patch.crates-io]` in `Cargo.toml`). That pulls in `cyw43` with `bt-hci` 0.9 and requires a few API updates in `main.rs` (dual DMA channels, `Cyw43439` chip type on `Runner`) - see the comments there.

## Setting Your Own Device Name

Since everyone in the workshop will be running the same code, by default every Pico would advertise under the same device name, making it hard to tell your device apart from your neighbours'.
To avoid that, before the program will compile you'll need to provide a short unique identifier through the `BLE_NAME` environment variable when compiling the program.
The firmware turns this into the advertised device name `LED <BLE_NAME>` - for example, `BLE_NAME=Alice` gives **LED Alice**, and `BLE_NAME=7` gives **LED 7**.

Pick something you will recognize in the scan list on your laptop or phone.
The name must fit into a single BLE advertisement packet, so keep `BLE_NAME` short (roughly 17 characters or fewer).

Like the WiFi credentials from the previous exercise, export `BLE_NAME` before building or flashing:

```shell
export BLE_NAME=Alice
```

## Testing over Bluetooth

Once your program is running, the Pico 2 W will advertise itself over Bluetooth Low Energy (BLE) under the name you configured — for example **LED Alice** if you set `BLE_NAME=Alice`.
You can connect to it from your laptop or phone and interact with the LED through GATT characteristics, in addition to the WiFi webserver from the previous exercise.

To do so, you will need a so-called _GATT client_ — a small application that can scan for BLE devices, connect to one, browse its services and characteristics, and read from or write to those characteristics.
We recommend installing one before the workshop so you can focus on the exercise itself.

### Setting Up a GATT Client on Your Laptop

For testing from your laptop, we recommend [toolBLEx](https://github.com/emericg/toolBLEx/releases), a cross-platform desktop application that uses your computer's built-in Bluetooth adapter.
Download the latest release for your operating system from the project's GitHub releases page:

- **Windows:** `toolBLEx-*-win64.exe` or `.zip`
- **macOS:** `toolBLEx-*-macOS.zip`
- **Linux:** `toolBLEx-*-linux64.AppImage` (or `.tar.gz`)

No extra hardware (such as a Nordic Bluetooth dongle) or programming environment is required — only a laptop with a working Bluetooth adapter.

Before the exercise, open toolBLEx once and start a scan to confirm that your laptop can see nearby BLE devices.
If the scan list stays empty, fix your host Bluetooth setup first (see the troubleshooting section below).

#### Windows

Make sure Bluetooth is turned on in the system settings.
If a connection to the Pico fails, try removing any stale pairing for the board in **Settings → Bluetooth & devices** and connecting again from toolBLEx.
Allow Bluetooth access if Windows prompts you when toolBLEx starts.

#### macOS

On first launch, grant toolBLEx permission to use Bluetooth under **System Settings → Privacy & Security → Bluetooth**.
If macOS blocks the app because it was downloaded from the internet, you may need to right-click the application and choose **Open** once to bypass Gatekeeper.
Note that macOS may show randomly generated device identifiers instead of MAC addresses — identify your Pico by its advertised name rather than by address.

#### Linux

Ensure the Bluetooth service is running (for example, `bluetoothctl show` should report a working adapter).
Your user account typically needs permission to talk to BlueZ; on many distributions this means being a member of the `bluetooth` group, followed by logging out and back in.
If you use the AppImage, make it executable (`chmod +x toolBLEx-*.AppImage`) before running it.

### Alternative: nRF Connect for Mobile

If you prefer to test from a phone, or if you run into problems with your laptop's Bluetooth stack, you can use [nRF Connect for Mobile](https://www.nordicsemi.com/Products/Development-tools/nrf-connect-for-mobile) instead.
It is available for free on [Android](https://play.google.com/store/apps/details?id=no.nordicsemi.android.mcp) and [iOS](https://apps.apple.com/app/nrf-connect-for-mobile/id1054362403) and offers the same basic workflow: scan, connect, browse the GATT table, and read or write characteristics.

### What to Look For on the Device

When the Pico is advertising and ready to accept connections, you should be able to find a device with the following properties:

| What | Value |
| --- | --- |
| Advertised name | `LED <BLE_NAME>` (e.g. `LED Alice`) |
| Service UUID | `0x180A` (Device Information — reused here for the LED service) |
| Characteristic UUID | `0x2A57` (`color`; supports read, write, and notify) |
| Valid write values | Single byte: `0x00` = red, `0x01` = green, `0x02` = blue |

Reading the characteristic returns the current LED color as one of those three byte values.
Writing a single byte in that range changes the LED; any other value is rejected as out of range.

### Using toolBLEx

With your program running on the Pico, open toolBLEx and start a scan.
When your device (for example **LED Alice**) appears in the device list, connect to it and open the GATT table.
Navigate to service `0x180A` and the characteristic `0x2A57`.

You can read the characteristic to see the current color (`00`, `01`, or `02` in hexadecimal).
To change the LED, write a single byte to the characteristic: `00` for red, `01` for green, or `02` for blue.
If you enable notifications on the characteristic, you should also see updates when the LED color changes through another path (for example, via the WiFi webserver).

The workflow in nRF Connect for Mobile is the same: scan for your device name, connect, expand service `0x180A`, and use the read, write, and notify controls on characteristic `0x2A57`.

### Troubleshooting

If your device does not show up in the scan list, check that the Pico is powered, your program is running, and Bluetooth is enabled on your laptop.
Also confirm that you are looking for the name matching the `BLE_NAME` you set when building the firmware, not another participant's device.
Move closer to the board and make sure no other application is holding on to the Bluetooth adapter.

If the connection drops immediately after connecting, try power-cycling the Pico.
On Windows, removing a stale pairing for the device and reconnecting often helps.

If a write appears to succeed but the LED does not change, double-check that you are writing exactly one byte with a value of `00`, `01`, or `02`.
Other payloads are rejected by the firmware.

If the advertised name is missing and you only see a device address, look for a peripheral that advertises service UUID `180A`, or power-cycle the Pico and scan again.
