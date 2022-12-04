# About Hackdose

This is the hackdose project, a project for making better user of your micro solar power plants.
The idea is simple: If there is more solar energy than needed, store it in batteries (of devices you 
use everyday), or fridges, or
just use it (e.g. run your dish washer). It is not my idea, but I didn't find an implementation for my
needs (most importantly, inexpensive hardware).

# Requirements

What you basically need

 * a raspberry pi or Mango PI
 * a smart meter that its compatible with this software (if it is not, file an issue)
 * HS-100 smart plugs (or suitable opto-isolators)
 * a micro solar power plant

The technical principle is easy: if you produce more energy than your house needs, your 
smart meter will know and tell hackdose. Hackdose will (under some conditions) turn
on a smart plug with your charger (or whatever device you want to control).

# Warning

This project is experimental. Use at your own risk. Risks may include

 * this software is able to control devices with high power. It can cause whatever eletrical devices can cause.
 * although by design there is no remote control of the software apart from IR possible, there is a rest interface which may potentially be subject to vulnerabilities

# Features

## Monitoring

 * there is a energy monitoring endpoint on port 8080 path `/energy`.
 * there is a 24h-statistics endpoint on `/day` showing a nice diagram

## Smart usage of energy

You can currently set up smart plugs (currently only HS-100) 
to prevent solar energy from escaping behind your smart meter.
You can e.g. use this to charge your Laptop battery, your smart phone, or your E-bike.

# Setup

You can either compile your yourself or check out one of the latest artifacts
[here](https://github.com/torfmaster/hackdose-server/actions/workflows/release.yaml).

## Install compiler toolchain

```bash
rustup target add riscv64gc-unknown-linux-gnu
```

If you are using another architecture (e.g. arm), install an appropriate rust target
and set it up in `install.sh`.

## Install cross linker (for linking)
```bash
sudo apt install gcc-riscv64-linux-gnu
```

## install sshpass (for deployments)
```bash
sudo apt install sshpass
```

## Setup your Hardware

I assume that you have a Mango PI Pro-Q.

 * connect IR reader (using PIN 35 as power supply to avoid boot startup trouble)
 * install systemd config file (see sample)

## Setup actors

 * see sample yaml config
 * put your HS100/HS110 smart plugs into the list

# Deploy

You can use the deploy script `install.sh` to deploy. Make sure to set up an appropriate `.env` file
(sample included).

# License

This project is licensed under Apache 2.0 or MIT license.

# Contributions

Contributions are very welcome. Just file an issue (even if you just have an idea) or solve one.
