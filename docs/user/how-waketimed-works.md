[parent page](index.md)

# How waketimed works

## Overview

Waketimed is a daemon. It monitors various properties of the system,
and maintains a set of internal variables based on the observed state.
A set of *stay-up rules* is defined, each rule referencing one or more
of the variables. When at least one stay-up rule is evaluated as
active, waketimed does not attempt to suspend the system. When none of
the stay-up rules evaluates as active, waketimed suspends the system
after a short period of time.

> Note: In the future, waketimed may implement also *wake-up rules*,
> which would be able to use the same internal variables to
> dynamically compute desired periods in which the system would be
> woken up.

## Variable definitions

Variables are described in YAML format. Some variable definitions are
[built into the daemon executable](https://github.com/jistr/waketimed/tree/main/waketimed/embed/var_def),
and additional ones may be specified inside the configuration
directory, by default under `/etc/waketimed/var_def`.

See also [variables and rules](variables-and-rules/index.md).

## Rule definitions

Stay-up rules are described in YAML format. Some stay-up rule
definitions are
[built into the daemon executable](https://github.com/jistr/waketimed/tree/main/waketimed/embed/rule_def),
and additional ones may be specified inside the configuration
directory, by default under `/etc/waketimed/rule_def`.

See also [variables and rules](variables-and-rules/index.md).
