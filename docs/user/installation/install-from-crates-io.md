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
