[parent page](index.md)

# Compilation from source

These instructions will produce a binary for the architecture you're
compiling on.

* **Install compilation requirements.** The exact steps will vary per
  distro.

  On Mobian:

      ```
      sudo apt-get install cargo git make
      ```

* **Get the code.**

      ```
      git clone https://github.com/jistr/waketimed
      cd waketimed
      ```

* **Compile.**

      ```
      make
      ```

* **Install binary.**

      ```
      sudo make install
      ```

* **Install systemd service file, enable and start the service.**

      ```
      sudo make install-service
      sudo systemctl daemon-reload
      sudo systemctl enable waketimed.service
      sudo systemctl start waketimed.service
      ```
