
add stuff


# initial setup

__adapted from https://graspingtech.com/ubuntu-desktop-18.04-virtual-machine-macos-qemu/__

* install home brew if you don't already have it
* install qemu with `brew install qemu`
* check that it works with `qemu-system-x86_64 --version`
* go into qemu dir `cd qemu`
* first create the server.qcow2 with `qemu-img create -f qcow2 server.qcow2 10G`
* also download the torrent of the ubuntu iso. probably from [here](https://ubuntu.com/download/server)
* edit `run_qemu_x86_64.sh` to point `server.qcow2` and the iso file.
* run `./run_qemu_x86_64.sh`. choose all defaults. no extra snaps or updates.
* after install control-c to stop it
* then run again without the -cdrom line
* now log into the virtual server
* you can ssh to the virtual server with  `-net user,hostfwd=tcp::2222-:22 -net nic \` added to the `./run_qemu_x86_64.sh` file.
   *    
* install some deps: `sudo apt-get install curl git build-essential make gcc -y`
* now install rust and cargo. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* now log out and back into the virtual server
* now check out the code with `git clone https://github.com/joshmarinacci/clogwench.git`
* now run `cargo build`
* now run `cargo run` it will probably fail if you don't have root privs. add privs with
* `sudo usermod -aG video <username>`
* `sudo usermod -aG input <username>`


* You can run it on a real raspberry pi by following the same steps, just without the QEmu parts. Once you can SSH into your pi (or do it directly on the device), run the same steps to install rust and the source, add your user to the root privs, then start it.

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



plan for platform independence

create plat crate. provides
  init_plat() returns a Plat object
  get_screen() a screen object with key fields on it
  get_input_stream() which is a channel that returns incoming messages
  plat.stop() shuts down everything, including any internal threads

create plat-linux crate: uses my kernel work

create plat-mac crate: uses sdl
