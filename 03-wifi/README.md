# Making Your Board Detect How Close You Are

TODO

## Nightly Rust

We'll be using the [`picoserve` crate](https://docs.rs/picoserve/0.17.1/picoserve/) to implement a tiny web server on the Pico 2.
Since `picoserve` uses one particular unstable Rust feature, you'll need to use a nightly Rust compiler for this exercise.
We've pre-configured a known working compiler in the `rust-toolchain.toml` file, so no action is needed on your part - `cargo` should automatically notice if you're missing the required toolchain and download it for you.

## WiFi Credentials

If you're reading this, you're probably already connected to a WiFi network with your laptop.
You'll need to provide the name of the WiFi network (its SSID) and the WiFi password as environment variables when compiling the program so the Pico 2 will be able to connect to it.
To do so, please export the WiFi name as `SSID` and the WiFi password as `PASSWORD`.

## Wiring

_There are no wiring changes for this exercise._

## Coding

For this exercise, you'll have to do TODO things:

1. Finish the implementation of the `LedControllerRunner` responsible for blinking the LED and changing its color.
2. TODO

Usually the was this works on embedded or at least in `embassy` is that the IO resources are _driven_ by an `embassy` task.
That is, there's a task for every part of the stack: one for running the WiFi chip, one for running the network (TCP/IP) software stack from `embassy_net`, one for our own web server, etc.

You can use the provided `mk_static!` macro to safely create static mutable references (`&'static mut T`) to the value given to the macro.

Change the color with

```shell
curl -X POST <IP>/color/<color>
```

so e.g. you should be able to `192.168.92.85/color/blue` to change the LED to blue if that's your IP.
In your terminal, you should then see the response `{ "color": "blue" }`.

Hint: you can use [`AppRouter::shared`](https://docs.rs/picoserve/0.17.1/picoserve/type.AppRouter.html#method.shared) to go from a reference to a router to an owned router type. Make sure you also pass the `AppState` so the server has access to the color value channel.

```shell
0.000171 [INFO ] Initializing CYW43 
0.508436 [INFO ] Spawning CYW43 task 
1.243209 [INFO ] Initialized CYW43 
1.243228 [INFO ] Creating network stack 
1.243501 [INFO ] Created network stack 
1.243514 [INFO ] Initializing LED controller 
1.243536 [INFO ] Initializing web server 
1.243558 [INFO ] Spawning tasks 
1.244024 [INFO ] Tasks spawned 
1.244043 [INFO ] Joining network 
4.294398 [INFO ] Joined network 
4.745652 [INFO ] Waiting to get IP address... 
7.309739 [INFO ] Got IP: 192.168.92.85 
9.294448 [INFO ] Hello from main! 
```

If you don't see the IP address being printed, try running the program again (it's possible that we'll struggle with the workshop WiFi).
If you don't need the log output, you can also restart your program without waiting for it to be flashed again by running `probe-rs reset`.

<details>

<summary>Hint 1</summary>

Since we've wired up our I2C bus to GPIO pins 14 and 15, we're using the `I2C1` peripheral, which therefore needs to be set as the type parameter of the `InterruptHandler`.
You'll have to find the correct interrupt routine for `I2C1` in `embassy_rp::interrupt::typelevel`.

</details>

<details>

<summary>Hint 2</summary>

Take a look at the implementation of the `Instance` trait for the `I2C1` peripheral to find the name of the correct interrupt.

</details>

#### Bus Driver

With that out of the way, create an async instance of `embassy_rp::i2c::I2c` using the correct pins and other peripherals.
You can use the default I2C [`Config`](https://docs.rs/embassy-rp/0.9.0/embassy_rp/i2c/struct.Config.html) for the last parameter.

<details>

<summary>Hint 1</summary>

You want to use the [`new_async` constructor](https://docs.rs/embassy-rp/0.9.0/embassy_rp/i2c/struct.I2c.html#method.new_async) method on `I2c`.

</details>

<details>

<summary>Hint 2</summary>

You'll have to pass the correct peripheral again, as well as the SCL and SDA pins and the name of the struct you created inside `bind_interrupts!`.

</details>

### Sensor Setup

Now it's time to connect the actual sensor!
Create an instance of the `Apds9960` driver and use it to initialize the connected sensor.
You can find the documentation of the `apds9960` crate [here](TODO: link APDS docs).

<details>

<summary>Hint</summary>

The sensor initially starts out in a power-saving sleep mode.
You need to call `enable()` on it once to get it to do anything.
You'll also need to separately enable the proximity function.

</details>

### Measuring Proximity

Modify the main loop to check if the sensor detects something in its proximity.
Depending on how close the detected object is, have the LED turn on in green first, then switch to yellow as the object comes closer, and finally it should show red if the object is very close.

<details>

<summary>Hint</summary>

Try matching on the result of the sensor's `read_proximity()` method.

</details>

Depending on the conditions around you in the room and also the sensor itself, each sensor might report slightly different proximity values for a specific distance.
Feel free to experiment with the exact thresholds a bit to find out what feels right for your sensor.
If you're unsure, it should be a reasonable starting point to detect a presence if the proximity is at least 2, switch to yellow at around 10 and to red at around 200.
