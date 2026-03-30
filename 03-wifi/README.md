# Overloading the Workshop WiFi

Now for something slightly different: as our final exercise (for now), we'll make use of the on-board WiFi chip to turn your Pico 2 into a minimal webserver.
At the end of the exercise, you will be able to send a POST request from your laptop to the Pico 2 to change the color of the LED and receive a JSON response as a confirmation.

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

For this exercise, you'll have to do 4 things:

1. Implement a `TriColorLed` that keeps track of which color it is currently blinking.
2. Complete the request handler for the POST request and the initialization function of the webserver.
3. Finish the implementation of the `LedControllerRunner` responsible for blinking the LED and changing its color.
4. Initialize everything in `main`, join the WiFi network and start the webserver.

### Setting Up the LED Abstraction

Our first step is small: to make your life easier in the next step, complete the `toggle` and `set_color` functions on `TriColorLed`.
The `TriColorLed` type is a small wrapper around the three `Output` pins for red, green and blue that keeps track of the current LED color and state (on or off).

<details>

<summary>Hint</summary>

Make sure the LED for the previous color is turned off before switching on the new color to avoid color artifacts.

</details>

### Building a small Webserver

Next, we'll implement handling incoming POST requests with `picoserve`.
Each POST request will be handled by one of our `WebserverRunner`s.
These have access to the `embassy` net stack, the `picoserve` router that routes requests to the correct handler, the configuration to use for `picoserve` and our own `AppState`, which contains a channel that you can send new color values to if they are requested.

Complete the `run` method of the `WebserverRunner` by creating a `picoserve::Server` and making it listen for incoming requests.

<details>

<summary>Hint</summary>

Hint: you can use [`AppRouter::shared`](https://docs.rs/picoserve/0.17.1/picoserve/type.AppRouter.html#method.shared) to go from a reference to a router to an owned router type. Make sure you also pass the `AppState` so the server has access to the color value channel.

</details>

You'll also need to complete `build_app` to actually do something when a POST request is received.
Forward the requested value to the LED controller and return a JSON response with a single field `color` that contains the new color, like `{ "color": "red" }`.

### Implementing the LED Controller

Now, finally, you need to actually handle the new color value and set it on the LED.
To do so, start by implementing the `initialize` function which initially creates the controller.
You'll need to create a `TriColorLed` with the correct pins and the channel that connects the LED controller to the webserver.

> [!TIP]
> You can use the provided `mk_static!` macro to safely create static mutable references (`&'static mut T`) to the value given to the macro.

Once created, the main program will call the LED controller's `run` method, which is responsible for both making the LED blink on and off as well as changing its color when new color values come in on the channel. 
Find a way to simulataneously wait for incoming color values on the `ColorReceiver` while also regularly blinking the LED on a timer.

<details>

<summary>Hint 1</summary>

Have a look at the [`embassy_futures`](https://docs.rs/embassy-futures/0.1.2/embassy_futures/) crate for some helper functions for working with multiple futures.

</details>

<details>

<summary>Hint 2</summary>

The `select` function is what you want here: if you pass both `self.receiver.changed()` and `ticker.next()`, you can `match` on which of the two happens next.

</details>

### Bringing It All Together

The final thing we need to do is to create and run all of the top-level networking resources.
Usually, the way this works on embedded or at least in `embassy` is that the IO resources are _driven_ by an `embassy` task.
That is, there's a task for every part of the stack: one for running the WiFi chip, one for running the network (TCP/IP) software stack from `embassy_net`, one for our own web server, one for the LED controller, etc.
You can see these tasks pre-created for you at the top of `main.rs`.

The lower level parts are already set up in `main`.
Finish up the main function by initializing the LED controller and the webserver and making the Pico 2 join the WiFi network through the WiFi `control` with the provided `SSID` and `PASSWORD`.

<details>

<summary>Hint</summary>

Joining the network is as simple as calling `Control::join`.
There should be no need to edit the default `JoinOptions`, passing the `PASSWORD` to `new` should be sufficient to establish a connection.

</details>

## Testing Your Webserver

When you now run your program, you should see some log output like this:

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

You should then be able to change the color of your LED by running

```shell
curl -X POST <IP>/color/<color>
```

so e.g. posting to `192.168.92.85/color/blue` should change the LED to blue if that's your IP.
In your terminal, you should then see the response `{ "color": "blue" }`.


> [!TIP]
> If you don't need the log output, you can also restart your program without waiting for it to be flashed again by running `probe-rs reset`.
