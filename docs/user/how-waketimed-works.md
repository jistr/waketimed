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

Some variable definitions are built into the daemon executable. While
their
[descriptions are in YAML format](https://github.com/jistr/waketimed/tree/main/waketimed/embed/var_def),
the actual
[variable poll code is compiled from Rust](https://github.com/jistr/waketimed/tree/main/waketimed/src/var_fns/poll)
and executed on a separate small thread pool through asynchronous runtime,
making the polls performant yet light on resources.

Additional variables may be specified inside the configuration
directory, by default under `/etc/waketimed/var_def`. Currently this
is not very useful, because scripting or external program execution
from variables are not implemented.

See also [variables and rules](variables-and-rules/index.md).

## Rule definitions

Stay-up rules are described in YAML format. The rules are evaluated
using [Rhai](https://rhai.rs/) expressions/scripts which reference one
or more of the waketimed variables.

Some stay-up rule definitions are
[built into the daemon executable](https://github.com/jistr/waketimed/tree/main/waketimed/embed/rule_def),
and additional ones may be specified inside the configuration
directory, by default under `/etc/waketimed/rule_def`.

See also [variables and rules](variables-and-rules/index.md).
