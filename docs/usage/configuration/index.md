# Configuration

Waketimed attempts to load its configuration during startup from **a
file and/or environment variables**. Environment variables take
precedence over the configuration file. If a configuration option is
not set via either a file or an environment variable, default value
(compiled into waketimed) is used.

**Waketimed should operate sanely even in absence of any configuration
file / environment variables.**

## Configuration file

If a configuration file is used, it must be valid YAML. The file may
contain an incomplete set of config options.

* The default location is `/etc/waketimed/config.yaml`. Waketimed will
  start normally even if the file does not exist.

* The `WAKETIMED_CONFIG` environment variable can be used to set
  configuration file location explicitly. When specified this way, the
  file must exist.

## Configuration options

### General

* `log` – Log level. Available log levels are: error, warn, info,
  debug, trace. Can optionally contain per-module log level setting,
  syntax for this can be found in
  [env_logger documentation](https://docs.rs/env_logger/0.9.0/env_logger/).

  Type: string  
  Default: `"info"`  
  Environment variable: `WAKETIMED_LOG`

* `state_dir` – Directory customizable by the admin and writable by
  the waketimed daemon. Contains custom rule and variable definitions,
  and in the future may contain daemon state.

  Type: string  
  Default: `"/var/lib/waketimed"`  
  Environment variable: `WAKETIMED_STATE_DIR`

* `dist_dir` – Directory with files distributed and upgraded together
  with the waketimed daemon. Contains built-in variable and rule
  definitions. Should be treated as read-only except for waketimed
  installation/upgrade procedure.  

  Type: string  
  Default: `"/usr/lib/waketimed"`  
  Environment variable: `WAKETIMED_DIST_DIR`

* `allowed_chassis_types` – List of 
  [chassis types](https://www.freedesktop.org/software/systemd/man/machine-info.html#CHASSIS=)
  for which waketimed should normally operate. If waketimed is
  launched on a device whose chassis is not in the list, it enters
  disabled mode - it doesn't load any rules/variables and it stays
  passive until terminated. Recognized values are: desktop, laptop,
  convertible, server, tablet, handset, watch, embedded, vm,
  container.

  This can be useful in case waketimed is included in a Linux distro
  image which is common for all chassis types, but it should only
  operate on chassis types where it makes sense.

  Type: list of strings  
  Default: `["convertible", "embedded", "handset", "tablet", "watch"]`  
  Environment variable: `WAKETIMED_ALLOWED_CHASSIS_TYPES`

### Timing

* `poll_variable_interval` – Time between polls of poll-based
  variables, in milliseconds. Larger values mean less frequent
  variable updates and e.g. less exact times when device falls asleep.

  Type: integer  
  Default: `3 000` (= 3 seconds)  
  Environment variable: `WAKETIMED_POLL_VARIABLE_INTERVAL`

* `startup_awake_time` – Minimum time in milliseconds for which
  waketimed shouldn't be putting the device to sleep after waketimed
  starts.

  This is useful to let the device complete the full start up sequence
  before the first suspend. It can also be useful in case waketimed is
  somehow misconfigured and would be too eager to put the device to
  sleep - it gives admin the chance to stop waketimed before it tries
  to act.

  Type: integer  
  Default: `300000` (= 5 minutes)  
  Environment variable: `WAKETIMED_STARTUP_AWAKE_TIME`

* `minimum_awake_time` – Minimum time in milliseconds for which the
  device should stay awake after it has woken up.

  > ⚠ Waketimed currently lacks detection of last wake up time, so
  > this configuration setting has no effect for now.

  Type: integer  
  Default: `10000` (= 10 seconds)  
  Environment variable: `WAKETIMED_MINIMUM_AWAKE_TIME`

* `stayup_cleared_awake_time` – Time in milliseconds for which
  waketimed will wait before suspending the device after the last
  stay-up rule became inactive.

  Type: integer  
  Default: `10000` (= 10 seconds)  
  Environment variable: `WAKETIMED_STAYUP_CLEARED_AWAKE_TIME`

### Testing

* `test_mode` – When `true`, waketimed will operate as normal except
  it will not put the device to sleep.

  Type: boolean  
  Default: `false`  
  Environment variable: `WAKETIMED_TEST_MODE`
