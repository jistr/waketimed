# Included variables

These are variables built into waketimed, available for use within
rule definitions.

# Category boolean variables

Category boolean variables group one or more other boolean variables
together. The category variable is `true` when any variable within the
category is `true`, otherwise the category variable is `false`.

* `wtd_call_present: bool` – `true` if the device has any ongoing or
  ringing call. This category can group call status from multiple
  sources.

* `wtd_user_busy: bool` – `true` if the user is interacting with the
  device in some way.

# Leaf variables

These "leaf" variables are set based on inspection of the device
state.

* `wtd_login_seat_busy: bool` – `true` when login manager's seat0 is
  not idle (mainly when the phone screen is on). Included in category
  `wtd_user_busy`.

* `wtd_modem_voice_call_present: bool` – `true` when the device's
  modem manager tracks any voice calls (the device has an ongoing or
  ringing voice call). Included in category `wtd_call_present`.

* `wtd_sleep_block_inhibited: bool` – `true` when login manager's
  `BlockInhibited` property includes `sleep`.
