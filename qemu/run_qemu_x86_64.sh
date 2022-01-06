

#adapted from https://graspingtech.com/ubuntu-desktop-18.04-virtual-machine-macos-qemu/
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


# first create the server.qcow2
# also download the torrent of the iso image
# run it. choose all defaults. no extra snaps or updates.

