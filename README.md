[![crates.io](https://img.shields.io/crates/v/waketimed.svg)](https://crates.io/crates/waketimed)
[![license](https://img.shields.io/crates/l/waketimed.svg)](https://github.com/jistr/waketimed/blob/main/LICENSE)
[![test suite](https://github.com/jistr/waketimed/actions/workflows/test_suite.yaml/badge.svg?branch=main)](https://github.com/jistr/waketimed/actions/workflows/test_suite.yaml)

# waketimed

## Project goal

**Waketimed is an experimental daemon for managing the sleep/wake
cycle of Linux phones and similar devices. It aims to keep the device
sleeping as much as possible to preserve battery life.**

## Implementation status

When using waketimed currently, please be aware that **it's in early
development phase**. The design and feature set are being figured out.
Seamless non-breaking upgrades (e.g. config file compatibility between
versions) are a nice to have but not the main priority right now.

The project idea is that waketimed could put the device to sleep and
also wake it up periodically. However, only putting the device to
sleep is implemented right now. If you want periodic system wake-ups
in addition to wake-ups caused by the user or the modem, it is
presently recommended to create a systemd timer.

## Documentation

**[waketimed user documentation](https://github.com/jistr/waketimed/blob/main/docs/user/index.md)**

## Context

Without waketimed, Linux phones typically rely on automatic suspend
functionality inherited from desktop and laptop computers. While this
is a working solution, having the auto-suspend timer at
e.g.&nbsp;3&nbsp;minutes often leaves the phone idle but not suspended
for longer time than necessary, shortening precious battery life.
Setting the idle and suspend timers to something more aggressive like
20 seconds is possible, but it is disturbing when the user wants to
read something longer on the screen. Additionally, when testing these
short timeouts, they seemed somewhat unreliable in actually suspending
the device after the specified time.

Waketimed allows you to have generous idle timers in the desktop
environment, but once it detects the device is not being used (screen
off, no active/ringing call etc.), it suspends the device after a
rather short time (10 seconds by default).
