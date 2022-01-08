

#adapted from https://graspingtech.com/ubuntu-desktop-18.04-virtual-machine-macos-qemu/
qemu-system-x86_64 \
 -m 4G \
 -vga std \
 -display default,show-cursor=on \
 -usb \
 -device usb-tablet \
 -machine type=q35,accel=hvf \
 -smp 2 \
 -drive file=server.qcow2,if=virtio \
 -net user,hostfwd=tcp::2222-:22 -net nic \
 -cpu Nehalem
# -cdrom ubuntu-20.04.3-live-server-amd64.iso \
# -device usb-mouse \


# first create the server.qcow2
# also download the torrent of the iso image
# run it. choose all defaults. no extra snaps or updates.
# now install rust and cargo
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# now run cargo build
# now run cargo run
