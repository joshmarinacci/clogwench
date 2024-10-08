

# new instructions June 2022


### macos

First pre-install homebrew and rust, then install `pkg-config` and `sdl2`

```shell
brew install pkg-config
brew install sdl2
```

now build everything and run it
```shell
cargo build
cargo run-script dev
```

An SDL window will open with a mouse cursor and a pink-magenta button to quit.

In another terminal window start the dock with `cargo run-script dock`



### linux

* install rust and gcc and git and node
  * [follow these instructions](https://www.rust-lang.org/tools/install)
  * `sudo apt-get install build-essential`
  * [install nodejs v16](https://github.com/nodesource/distributions/blob/master/README.md) 


Now you need the following native deps:
* `sudo apt install libasound2-dev`

Now build IdealOS and run it.
```shell
cargo build
cargo run --bin runner -- --wmtype native --datafile=tools/runner/data.json
```


# Apps

This `clogwench` repo is just the core and window manager. The apps are in 
the [clogwench-app](https://github.com/joshmarinacci/clogwench-apps) repo. To use the apps, clone
that repo, build them with the instructions, then add the `apps.json` file to the `datapath` when
you start IdealOS. For example: if  you checked out `clogwench-apps` next to `clogwench` then you
can run:

```shell
cargo run --bin runner -- --wmtype native --datafile=tools/runner/data.json --datafile=../clogwench-apps/apps.json
```

You may also want the test data

```shell
git clone https://github.com/joshmarinacci/querylang-testdata.git
```

This will let you have cool sample music, images, and text documents to work with. To run the test data
as well as the apps, run:

```shell
cargo run --bin runner -- --wmtype native --datafile=tools/runner/data.json --datafile=../clogwench-apps/apps.json --datafile=../querylang-testdata/data.json
```



## details

### Running components separately.

In one terminal run the central server

```shell
cargo run --bin central -- --database = db/test_data.json
```

In another terminal go to the `clogwench-apps` repo and run an app:

```shell
cd ../clogwench-apps/dock
npm install
npm run dev
```

And the dock will launch. It will check the database for the list of
available apps and create a button for each one.

# arch

diagram [diagram](./tools/docs/diagram.md)



# old stuff
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

for the mac simulator pre-install  sdl2 and sdl2-image with

`brew install sdl2 sdl2_image`



To get this running on your real pi, get rust and git on your Pi then
ssh in and check out the code. Inside the clogwench repo do:

``` shell
cargo build
cd devtools
cargo run -- --debug=true --disable-network=true
```

This will start the window manager and test out graphics with a fake app.  To use a real central
server use `--disable-network=false`  
then start a demo app from another ssh session.




the Qemu screen is RGBA  but my standard 32bit is ARGB.
The gfxbuffer format needs to account for bit length and order of components.

