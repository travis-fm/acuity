# Acuity Hardware Monitor

A WIP TUI-based hardware monitor, with HwInfo64 on Windows as inspiration.

This is mainly a learning project for me to get more familiar with Rust, but I also plan on hacking
away at this thing long-term to turn it into a full-featured, easy to use monitor. We'll see how that actually
goes 😊.

It only supports Linux HWMon-based sensors right now, but next up will be AMD Ryzen CPU sensors.

Overall Goals:

* Easily extendable to get information from custom module drivers
* Customize module and sensor names, values, etc.
* Log, view, and chart sensor values over time
* Other stuff that I'm probably forgetting
