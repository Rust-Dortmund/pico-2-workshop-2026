# Making Your Board Detect How Close You Are

Now for something a little more involved: we'll use the provided APDS9960 light, color and gesture sensor to make the Pico 2 notice your presence. And we'll have it light up the LED as a little traffic light to show you when you start getting too close for comfort.

## Introducing: The APDS960

The APDS9960 sensor

You can find the full datasheet for the sensor [here](../datasheets/apds9960.pdf).

We're using a breakout board

## Wiring

### Background: The I2C Bus

_You may skip this section if you just want to continue with the workshop exercises - we'll be using a pre-existing library to talk to the APDS sensor, so this section is only for explaining what you are wiring up.

### Wiring Instructions

> [!TIP]
> If you're using the [online Pico 2 pinout](https://pico2.pinout.xyz/) shown in the previous exercise, you can turn on the "I2C" checkbox at the top to see which pins can be used for I2C communication.

<div align="center">

<img alt="Wiring Diagram" src="Wire_APDS.png" width="50%" />

</div>

> [!NOTE]
> We couldn't find a schematic with the exact visuals as our breakout board, so the diagram above shows a slightly different board for the APDS9960.
> `SDA`, `SCL` and `GND` are the same on our sensors.
> The fourth pin, labelled `3Vo` in the diagram, is labelled `VCC` for us - it's the only one connected to the top-right of the Pico 2 and the only remaining free pin that is part of the same group / side as the other 3 on our APDS board.

## Coding

### I2C Setup

TODO: link to `embassy_rp` docs.

#### Interrupt

Bind the interrupt using macro

Hint: we use bus I2C1 ()

#### Bus Driver

Create an async instance of `embassy_rp::i2c::I2C` using the correct pins and other peripherals.
You can use the default I2C config for the last parameter.

Hint: You'll have to pass the correct peripheral again, as well as the name of the struct you created inside `bind_interrupts!`.

### Sensor Setup

Create an instance of the `Apds9960` driver and use it to initialize the connected sensor.
TODO: link APDS docs

Hint: The sensor initially starts out in a power-saving sleep mode.
You need to call `enable()` on it once to get it to do anything.
You'll also need to separately enable the proximity function.

### Measuring Proximity

Modify the main loop to check if the sensor detects something in its proximity.
Depending on how close the detected object is, have the LED turn on in green first, then switch to yellow as the object comes closer, and finally it should show red if the object is very close.

Hint: try matching on the result of the sensor's `read_proximity()` method.

Depending on the conditions around you in the room and also the sensor itself, each sensor might report slightly different proximity values for a specific distance.
Feel free to experiment with the exact thresholds a bit to find out what feels right for your sensor.
If you're unsure, it should be a reasonable starting point to detect a presence if the proximity is at least 2, switch to yellow at around 10 and to red at around 200.
