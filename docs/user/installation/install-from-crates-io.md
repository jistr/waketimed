[parent page](index.md)

# Installation from crates.io

* **Install cargo.** The exact steps will vary per
  distro.

  On Mobian:

  ```
  sudo apt-get install cargo
  ```

* **Install the waketimed package.** This installs waketimed among
  current user's cargo binaries.

  ```
  cargo install waketimed
  ```

  > Note: `cargo install` works by compiling the binary. On a device
    like PinePhone, this can take a while to finish. Additionally,
    some Rust compilation artifacts will be under `~/.cargo`
    directory. Once you've finished the whole waketimed installation
    procedure, you can delete the `~/.cargo` directory if you wish to
    reclaim disk space.

* **Install the executable system-wide.**

  ```
  sudo install -m 0755 ~/.cargo/bin/waketimed /usr/local/bin/waketimed
  ```

* **Install systemd service file, enable and start the service.**

  ```
  curl https://raw.githubusercontent.com/jistr/waketimed/main/waketimed/config/systemd/waketimed.service \
    | sudo tee /etc/systemd/system/waketimed.service
  chmod 0644 /etc/systemd/system/waketimed.service

  sudo systemctl daemon-reload
  sudo systemctl enable waketimed.service
  sudo systemctl start waketimed.service
  ```
