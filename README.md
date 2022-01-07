add stuff


# initial setup

__adapted from https://graspingtech.com/ubuntu-desktop-18.04-virtual-machine-macos-qemu/__

* first create the server.qcow2
* also download the torrent of the ubuntu iso. probably from [here](https://ubuntu.com/download/server)
* run `qemu/run_qemu_x86_64.sh`. choose all defaults. no extra snaps or updates.
* after install control-c to stop it
* then run again without the -cdrom line
* now log into the virtual server
* install some deps: `sudo apt-get install curl build-essential make gcc -y`
* now install rust and cargo. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* now log out and back into the virtual server
* now run `cargo build`
* now run `cargo run` it will probably fail if you don't have root privs.


`sudo usermod -aG video <username>`
`sudo usermod -aG input <username>`

```
qemu-system-x86_64 \
-m 4G \
-vga virtio \
-display default,show-cursor=on \
-usb \
-device usb-tablet \
-machine type=q35,accel=hvf \
-smp 2 \
-drive file=server.qcow2,if=virtio \
-cpu Nehalem
# -cdrom ubuntu-21.10-live-server-amd64.iso \
```






Now check out this repo on the emulated pi

https://github.com/joshmarinacci/clogwench.git



