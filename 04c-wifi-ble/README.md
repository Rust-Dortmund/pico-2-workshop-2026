# Bluetooth Low Energy

Most of you have probably used Bluetooth on your phone, PC, headphones, and various other appliances - it's a very widespread protocol for short-range wireless communication (somewhere in the middle between NFC and WiFi depending on configuration).
Naturally, this means that it and particularly its "Low Energy" variant ("BLE") are very useful for embedded devices as well, so in this example we'll extend the Pico 2's connectivity with BLE capabilities, allowing you to read and change the current LED color over Bluetooth in addition to WiFi.

> [!NOTE]
> Like the WiFi example, running this example requires a nightly Rust toolchain (for picoserve's ITAIT routing).

> [!NOTE]
> `trouble-host` 0.7 is pulled from crates.io. Published `cyw43` 0.7.0 still depends on `bt-hci` 0.8, which is incompatible with `trouble-host` 0.7 (`bt-hci` 0.9). This crate therefore patches the Embassy stack from a pinned [git revision](https://github.com/embassy-rs/embassy/commit/c708723686ce80f83c80b059f04446c732db7272) (see `[patch.crates-io]` in `Cargo.toml`). That pulls in `cyw43` with `bt-hci` 0.9 and requires a few API updates in `main.rs` (dual DMA channels, `Cyw43439` chip type on `Runner`) - see the comments there.

## Background: Bluetooth Low Energy

The Bluetooth protocol was introduced in 1998, only a couple of years after than the original version of HTTP/1 ([RFC 1945](https://datatracker.ietf.org/doc/html/rfc1945) or HTTP/1.0), which was published in 1996.
Bluetooth operates over radio waves around 2.4GHz and data is exchanged over a packet-based protocol with a master/slave architecture.
In the terms of the frameworks we will be using today, what we typically think of as a Bluetooth device - the device that we connect to - is called a "peripheral" and the device we connect from (such as a laptop or phone) is called a "central".

Even the original Bluetooth protocol already concerned itself with the energy consumption of Bluetooth devices, but generally expected devices to be continuously connected to each other and exchange data, as is the case with devices like headsets, wireless mice or keyboards, or other devices where the Bluetooth functionality primarily removes the need for them to be wired.
With the advent of ever smaller devices this was no longer sufficient, as they rarely have the need to continuously stream data, and have to consume even less power, especially if they are battery-powered.
Examples for this category are smartwatches, health / fitness trackers, smart home sensors, locks, and tracking beacons ("tags"). 
This led to the development of "Bluetooth Low Energy" ("BLE") as a new Bluetooth protocol focused on energy efficiency.

BLE uses the same radio frequencies as "classic" Bluetooth, which you will see in the code as Bluetooth BR ("basic rate") or EDR ("extended data rate"), and can run side-by-side with it.
However, not all devices have to support both and we will only be working with BLE today.
BLE achieves lower power consumption by two primary means:
First, BLE devices don't need to maintain direct connections to their peers continuously and can spend a lot of their time sleeping between data exchanges.
Second, BLE data can be broadcast without requiring a peer-to-peer connection to exist at all.
In cases where a direct connection has to be established, BLE also allows such a connection to be established faster because only a subset of the overall frequency range is used for this purpose.

### Host Controller Interface

Bluetooth implementations are usually split between a **controller** and a **host**.
You can think of the controller as the hardware components required to implement Bluetooth, such as antennas and components that convert analog radio signals to digital data, while the host is the software or application side of the implementation that handles encoding and decoding and decides what to send to which device when.

Our host implementation is [TrouBLE](https://embassy.dev/trouble/), a BLE implementation for the embassy stack.
TrouBLE can work with any controller that provides an implementation for [`bt_hci`](https://docs.rs/bt-hci/latest/bt_hci/), which the Pico's [CYW43439](https://docs.rs/cyw43/latest/cyw43/) WiFi and Bluetooth chip does.

### Advertisements

For Bluetooth devices to find one another, a Bluetooth connection is established as follows:

1. A peripheral device advertises its presence via specific advertising packets that exist exactly for this purpose,
2. The central scans for advertising packets to find nearby devices,
3. The central connects to a scanned device,
4. A direct channel is opened between the two devices.

#### Packet Format

We'll start by looking at the structure of a BLE packet and work our way down to advertising.
A BLE packet usually contains a small header including an address, a data payload called the "protocol data unit" or PDU, and a CRC checksum to catch transmission errors:

```
| Preamble | Address | Protocol Data Unit (PDU) | CRC Checksum |
|  1 Byte  | 4 Bytes |         2-39 Bytes       |    3 Bytes   |
```

The preamble is a fixed bit pattern that devices can check for to quickly find where a packet starts.
For each pair of devices or broadcast, a unique address is used for all packets that are part of that connection.
Note that this is different from something like a physical / MAC address, which belongs to a distinct device and would be the same for all of this device's connections - here, the address identifies the communication channel between both devices, but the same devices will send packets with different addresses when communicating with other peers.
The PDU, then, contains any dynamic data that might be different between one packet and the next.

Different types of packets have different PDUs.
For advertising, the PDU again consists of a header and a payload.
Here, the header indicates the type of advertisement (more on that below), some flags, and the total length of the payload.

```
|           Header         |               Payload               |
|  Type  |  Flags | Length |                 Data                |
| 4 Bits | 4 Bits | 8 Bits |              0-37 Bytes             |
```

> [!TIP]
> **Types of advertisements**
> The Bluetooth standard distinguishes quite a few different types of advertisements because devices have to indicate together with the advertisement whether one can connect to them or not, if so, whether that is true for everyone or whether they are only advertising to a specific set of peers, and whether they can be scanned for additional information without connecting.
> There are also special advertisement types for advertising on nonstandard frequency channels.
> We will only be using the most simple type of advertisement today, which lets anyone scan and connect to the device.

The payload itself consists of only two things: the device address, which this time around _does_ act like a MAC address and will be the same for all messages from the same device and a list of advertisement data.
We can send as many advertisement data values as we want and can fit into 31 bytes.

```
| Device Address |      Advertisement Data      |
|    6 Bytes     |           0-31 Bytes         |
|                | AD 0 | AD 1 |   ...   | AD N |
```

For each entry / value, we send the following three things:

- The length of the entry (excluding the length field),
- A type tag that identifies the type of value we're sending, and
- The value itself.

The type tags are predefined and refer to fixed quantities like the name of the device, flags, which services a device will offer (see below), etc.

```
| AD Length |  AD Type |    AD Value    |
|   1 Byte  | m Bytes¹ | Length-m Bytes |
```

For example, one of the data sets that we will be sending is the advertising flags, which is always 3 bytes (so length `2`), has the tag `BT_DATA_FLAGS` and a 1-byte bitfield of up to 8 flags.

¹ Most type tags are 1 Byte, but there are some that are longer.

### Profiles

In order to provide a specific functionality through Bluetooth, the peripheral device and central need to agree on a shared interface that assigns semantic meaning to data points exposed by the peripheral and controlled by the central.
For example, for your headphones to act as headphones, they need to somehow tell your phone that they support audio streaming, your phone has to understand how to send audio data, and also how to control the volume of the music being played or read the headphone's battery level.
In the Bluetooth standard, these common interfaces are called **profiles**.

A Bluetooth profile in the most general terms is a specification regarding an aspect of Bluetooth-based wireless communication between devices. 
Profile definitions are based on the Bluetooth Core Specification, but may optionally make use of additional protocols for specific functionality.
Device manufacturers then implement the respective profiles for their devices depending on the intended function of the device.   
There are many specific profiles that the Bluetooth group has defined over the years, such as profiles for audio, imaging, printing, health (e.g, blood pressure), fitness and activity (e.g., heart rate and navigation), and many more.

### GATT

One particularly important profile which we will make use of today is called the "Generic Attribute Profile", or GATT, for short.
As the name implies, GATT allows devices to freely define values and advertise them to other devices connecting to them through a dynamic discovery mechanism instead of a static, pre-defined list of specific values that have to be provided.
The GATT protocol then layers on a client-server model for reading and writing to such "attributes" through generic endpoints that operate on generic ID representations of the attributes that are called **handles**.

In GATT terms, a **client** (like your phone) connects to a **server** (like the Pico 2) and sends it GATT commands and requests.
The server receives and processes them and returns a response.
The values / quantities that the client reads or writes are called **characteristics** - in our case, this is the current or intended color of the LED.
Multiple characteristics can be grouped into **services**.
There is also metadata that can be attached to characteristics, such as their allowed value range, name, or unit.
Metadata is optional and each piece of metadata is called a **descriptor**.

Characteristics, services and descriptors are all attributes in GATT and each attribute is tagged with a unique ID for reference.
Usually, full UUIDs are 128 bits, but we'll be using shorter 16-bit IDs which are also supported.
Some IDs are predefined and are always used for specific purposes.
For example, you will see that the Pico 2 will show a "Generic Access" GATT service, which contains the device name and what is called the "appearance", which in our case will indicate that the device is an LED.

#### Notifications & Indications

In addition to reading from or writing to a characteristic, GATT also has support for clients to subscribe to a characteristic and receive updates when its value changes.
If the peripheral waits for an acknowledgement when sending an updated value, this is called an **indication**.
Otherwise, if the value is just broadcast without feedback ("fire & forget"), this is called a **notification**.

### Further Reading

Since we can't possibly cover the entire Bluetooth standard as just one of the parts of this workshop, the above materials cover everything you need to know to get through the exercises, but not everything there is to know about Bluetooth.
If you want to know what we've left out, expand the section below.

<details>

<summary>Left Out</summary>

Here's some further things to search for online:

- Direct communication channels (L2CAP / CoC) & streaming
- Frequency channels, channel hopping, modulation, and advertising channels
- Scan data and extended advertising packets, directed and non-connectable advertisements
- The exact packet format of all of the other packets and the meaning of the other bits and bytes
- Security, pairing, and bonding
- Random and resolvable device IDs
- GATT protocol details and service metadata (discover services, related characteristics, etc.)

If you want to read more about BLE, a good place to start is the Nordic Semiconductor DevAcademy, which hosts an [introductory course](https://academy.nordicsemi.com/courses/bluetooth-low-energy-fundamentals/lessons/lesson-1-bluetooth-low-energy-introduction/topic/what-is-bluetooth-le/) that covers a broad set of topics.

</details>

## Wiring

_There are no wiring changes for this exercise._

## Coding

For this exercise, you'll have to do 6 things:

1. Enable the LED to be controlled by more than 1 source
2. Create a GATT service representing the LED
3. Advertise the LED GATT service via Bluetooth
4. Handle GATT reads of the LED color characteristic
5. Handle GATT writes of the LED color characteristic
6. Add support for subscribing to the LED color characteristic

### Foundations

When we added the HTTP API for setting color values, we introduced some synchronization primitives for communicating LED color changes / requests.
Since we only needed them to send requests from the web server to the LED runner, they can currently only facilitate a single connection.

If we're now keeping the HTTP API and adding Bluetooth on top of it, that is no longer enough.
To enable sending data to the LED runner from an incoming Bluetooth connection, generalize the types in `led_receiver` to more than 1 participant. 
Keep in mind that we're adding both a new sender (incoming Bluetooth writes) _and_ a new receiver for Bluetooth subscriptions / notifications.
Therefore, with keeping WiFi, there are now 2 senders and 2 receivers in total.

Then, initialize the Bluetooth stack from `main` by fixing the `todo!` about calling `ble::initialize` with the correct parameters.

<details>

<summary>Hint</summary>

You may also need to generalize the `LedControllerRunner` and `BleConnectionRunner` and adjust a few callsites.

</details>

### GATT Service Definition

Annotate the `LedService` type to create a GATT service with a single `u8` characteristic for the LED color.
The characteristic should be readable, writable and support notifications.
Then, provide the LED service as the single service of our `ble::Server`.

Remember that, initially, the LED will be RED.
Your definition should include a name for the characteristic, as well as the range of valid values.
Give the service the ID `0x180A` and the color characteristic the ID `0x2A57`.

> [!TIP]
> You can find the TrouBLE documentation on GATT services [here](https://embassy.dev/trouble/#_defining_services).

<details>

<summary>Hint 1</summary>

Have a look at the `#[characteristic]` and `#[descriptor]` annotations from TrouBLE.

</details>

<details>

<summary>Hint 2</summary>

Set the characteristic to `0` initially to match the starting LED color.

</details>

<details>

<summary>Hint 3</summary>

You will need the `VALID_RANGE` and `MEASUREMENT_DESCRIPTION` characteristics.
For the description, supply `type = &'static str` to fix the value type.

</details>

### Advertisements

Next up, now that we have a BLE service, we need to advertise it and accept incoming connections from devices who have received those advertisements.
To do so, complete the `advertise` function on `BleConnectionRunner` by

1. Creating an [advertisement data packet](#packet-format) containing the following items and encoding it into the `advertiser_data` buffer (hint: look for advertising-related types in the [`trouble_host` docs](https://docs.embassy.dev/trouble-host/git/default/index.html)):
   1. The "generally discoverable" and "BR and EDR not supported" flags to indicate that the device will keep advertising even if no one connects, but only supports BLE.
   2. The list of services the device provides (note that the [byte order](https://en.wikipedia.org/wiki/Endianness) in BLE is little-endian, meaning the lower byte goes first).
   3. The provided device name.
2. Building an advertisement of the correct type containing the `advertiser_data` (note: you can pass an empty `scan_data` slice if required). The device should allow other devices to connect and scan it.
3. Having the `peripheral` broadcast that advertisement.
4. Accepting incoming connections and providing our GATT `server` to them.

<details>

<summary>Hint 1</summary>

The TrouBLE type to look at for advertisement data and its encoding is `AdStructure`.

</details>

<details>

<summary>Hint 2</summary>

The correct advertisement type to use is `Advertisement::ConnectableScannableUndirected`.

</details>

<details>

<summary>Hint 3</summary>

For advertising, have a look at the `advertise` method on `Peripheral`.

</details>

<details>

<summary>Hint 4</summary>

Have a look for methods on `Connection` that transform it into the `GattConnection` type to be returned.

</details>

> [!NOTE]
> At this point, you should be able to flash and run your code and use [your client](./BLE_TESTING.md) to scan for your Pico.
> It should show up in the list of devices and allow you to connect, but won't yet display any actual values.

### Handling GATT Reads

After establishing a connection, we need to allow the central device to read the LED color. 
In GATT, a request to read a characteristic is indicated by a `ReadEvent`, which we handle in `handle_gatt_read`.
There are four things you need to do to complete the event handler:

1. Check whether the request is actually for the LED color value, which you can do by accessing the characteristic through the `server`.
2. If so, read out the value.
3. Accept the event, which constructs the BLE response for the central device. 
4. Send the response.

<details>

<summary>Hint 1</summary>

To compare values, compare the attribute **handles** of the event and our characteristic.

</details>

<details>

<summary>Hint 2</summary>

Have a look at the available methods on our `server` that might be suitable for reading values.

</details>

### Handling GATT Writes

Your next task is to implement the same thing for GATT writes to allow changing the LED color via BLE.
This works roughly the same as for reads, except that 

1. There is now incoming data associated with the event which we need to map back to an LED color,
2. We might need to reject the event instead of accepting it if the requested `u8` value doesn't exist as a color, and
3. You need to forward correct color requests to the LED runner.

Implement the write functionality in `handle_gatt_write`.

> [!NOTE]
> You can try out the new functionality by sending a write request from your client.

<details>

<summary>Hint 1</summary>

You can use `WriteEvent::with_data` to access the write payload.
The first parameter passed to the closure is a byte offset and will always be 0.
The second parameter is the actual data, which you can match on.

</details>

<details>

<summary>Hint 2</summary>

Transmit valid color requests via the provided `sender`.

</details>

<details>

<summary>Hint 3</summary>

The correct error code to use is `AttErrorCode::OUT_OF_RANGE`.
Even if you reject the event, you will still need to send a (negative) response.

</details>

### Notifications

Lastly, we're going to enable the peripheral to send change notifications to the central.
Since the LED color may change independently from BLE requests (e.g., via the HTTP API), this functionality runs as a separate `notify_task` which you need to implement.

The notify task actually has two distinct functions:

1. Sending the color value at the time of connection once to provide an initial value to the client (even if the LED does not yet change).
2. Sending updates whenever the color is modified.

Both of these can be accomplished through the same notification API, which you can find on the characteristic, but one needs to run continuously while the other only runs once at the start of the connection.

> [!TIP]
> Use your client to subscribe to notifications.

## What's Next

Congratulations!
You've made it to the end of the connectivity path.

If this was the first path you completed, you can switch to the sensing path to learn more about connecting the Pico 2 to other sensors and performing gesture detection - to do so, start [here](../03s-sun-detector/README.md).
Otherwise, or if you prefer to not do more structured learning today, you are free to poke at the APDS-9960, further explore the connectivity features of the Pico 2 or whatever else you want to take a peek at.
If you want some ideas for what you can achieve with just the equipment you have, then for example you could

- Have WiFi and Bluetooth control different aspects of the LED (such as color vs. brightness)
  - Or, if you've done the sensing path, color vs. operating mode or color vs. threshold for distance, gesture detection, etc.
- Learn about Bluetooth security and add authentication for Bluetooth pairing (small tip: while we haven't provided you with any display or buttons, `defmt` log output can be a form of display too...)
- Explore the predefined set of GATT services / profiles. For example, try to build a HID (human interface device) such as a game controller reacting to gestures.

However you decide: We hope that you had a great experience and enjoyed the workshop so far, we are happy to have you here!
We also appreciate feedback, just talk to us!
