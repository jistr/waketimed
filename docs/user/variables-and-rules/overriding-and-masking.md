[parent page](index.md)

# Overriding and masking of variables and rules

The variable and rule definitions which are built into waketimed are
under the directory which waketimed config refers to as `dist_dir`. By
default, this is `/usr/lib/waketimed`, so variables are in
`/usr/lib/waketimed/var_def`, and rules are in
`/usr/lib/waketimed/rule_def`.

Waketimed also has a `state_dir`, by default `/var/lib/waketimed`,
which can also contain `var_def` and `rule_def` subdirectories. These
can serve the following purposes:

* defining custom rules and variables,

* overriding the definitions of built-in rules and variables,

* masking built-in rules and variables.

If there is a variable or a rule definition file under `state_dir`
with the same file name as a variable (or rule, respecitvely)
definition under `dist_dir`, the file `state_dir` takes precedence and
the one under `dist_dir` is not loaded.

## Overriding

The file behavior described above means that the variables and rules
distributed with waketimed can be overriden with custom definitions.
To override e.g. `wtd_login_seat_busy` variable, create a
`<state_dir>/var_def/wtd_login_seat_busy.yaml` file with a custom
variable definition.

Overriding of rules works analogically. To override `wtd_user_busy`
stay-up rule, create `<state_dir>/rule_def/wtd_user_busy.yaml`.

## Masking (disabling)

A special case of overriding is masking, which effectively disables a
variable/rule. The same file is created under `state_dir` just as when
overriding, but instead of providing your custom definition for the
variable/rule, the file is left empty. This will cause waketimed to
skip loading the variable/rule.

Variable and rule removal shouldn't be done by manually removing
definition files from under `dist_dir`, as this directory should stay
under the control of package management. It should be treated as
read-only by users.
