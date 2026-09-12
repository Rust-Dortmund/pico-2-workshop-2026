# Troubleshooting Common Issues

If something's not working for you, you can check if the problem you're having has come up often enough that we've added it to the table below. 
You can always reach out to the instructors if you need additional help or can't find your particular issue.

| Issue | Troubleshooting Steps |
| ----- | ---------- |
| LED colors are mixed up | Check the wiring of the LED and Pico 2 |
| `cargo` keeps trying to compile for your laptop / Linux instead of the Pico 2 | Check your `Cargo.toml` for any newly added dependencies that might require `std`. If you're trying to use `probe-rs`, make sure you're installing it as a binary and haven't added it as a depedency crate. |
| Sensor not responding | Check the wiring of the sensor, especially supply voltage and I2C bus |
| The Pico 2 does not connect to your mobile WiFi | Check your WiFi settings and make sure that you're using 2.4 GHz and not 5 GHz (some devices default to 5 GHz, but will usually have a setting for it, if you can't select the frequency explicitly, look for an option to improve compatibility, e.g. it is called "Maximize Compatibility" on iPhones). On iPhones make sure to "Allow Others to Join". |
