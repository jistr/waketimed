[parent page](index.md)

# How waketimed works

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
