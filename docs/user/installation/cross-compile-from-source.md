[parent page](index.md)

# Cross-compilation from source (aarch64 executable on x86_64)

These instructions will produce an aarch 64 binary when running on
x86_64 machine. The process relies on `podman` and a cross-compilation
container image based on Debian.

* **Get the code.**

  ```
  git clone https://github.com/jistr/waketimed
  cd waketimed
  ```

* **Prepare the cross-compilation environment.**

  ```
  make cross-toolbox-build
  make cross-prep-cargo
  ```

* **Cross-compile the executable.**

  ```
  ./cross-tbx make build-release-aarch64
  ```

* **Copy the executable and a service file onto the target aarch64
  device, and enable the service.** For example:

  ```
  scp target/aarch64-unknown-linux-gnu/release/waketimed root@my-phone-ip-address:/usr/local/bin/waketimed
  ssh root@my-phone-ip-address chmod 0755 /usr/local/bin/waketimed

  scp waketimed/config/systemd/waketimed.service root@my-phone-ip-address:/etc/systemd/system/waketimed.service
  ssh root@my-phone-ip-address systemctl daemon-reload
  ssh root@my-phone-ip-address systemctl enable waketimed.service
  ssh root@my-phone-ip-address systemctl start waketimed.service
  ```
