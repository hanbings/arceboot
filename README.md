# ArceBoot
Reuse [ArceOS](https://github.com/arceos-org/arceos) components to build a cross-platform operating system bootloader

# Build

```bash
# for serial output build:
$ make
$ make run-qemu

# for qemu virtual monitor:
# this may require a desktop system or graphical infrastructure 
# such as x11 forwarding configured on your host machine.
$ make EXACT_FEATURES="display"
$ make run-display
```