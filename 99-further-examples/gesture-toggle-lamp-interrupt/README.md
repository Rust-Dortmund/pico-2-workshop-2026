# Gestures with Interrupt

This example sets up the Pico 2 to read gesture data from the APDS sensor and make the LED change colors if it detects that you swipe your hand horizontally or vertically in front of the sensor.

In contrast to the `gesture-toggle-lamp` example, which uses synchronous APIs of the sensor, this example configures the APDS sensor to send a signal on its dedicated interrupt pin, which allows the main code to asynchronously wait for gestures instead of polling if a new gesture was detected.

> [!NOTE]
> This example requires wiring up the remaining pins of the APDS board.
