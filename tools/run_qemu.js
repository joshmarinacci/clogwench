import child_process from 'child_process'
import util from 'util'
const execp = util.promisify(child_process.exec)


const HOME=`${process.env['HOME']}`

const IMAGE='2020-02-13-raspbian-buster-lite'
const KERNEL='kernel-qemu-5.4.51-buster'
const PTB='versatile-pb-buster-5.4.51.dtb'
const TMP_DIR=`${HOME}/qemu_vms`
const IMAGE_FILE=`${TMP_DIR}/${IMAGE}.img`
const KERNEL_FILE=`${TMP_DIR}/${KERNEL}`
const PTB_FILE=`${TMP_DIR}/${PTB}`
const QEMU_SYS='qemu-system-arm'

async function run_qemu () {
    let args = [
        QEMU_SYS,
        "-cpu arm1176",
        "-m 256",
        "-M versatilepb",
        `-drive file=${IMAGE_FILE},if=none,index=0,media=disk,format=raw,id=disk0`,
        `-device 'virtio-blk-pci,drive=disk0,disable-modern=on,disable-legacy=off'`,
        `-net 'user,hostfwd=tcp::5022-:22'`,
        `-net  nic`,
        `-dtb ${PTB_FILE}`,
        `-kernel ${KERNEL_FILE}`,
        `-append 'root=/dev/vda2 panic=1'`,
        `-no-reboot`,
        // `-nographic`
    ]
    let cmd = args.join(" ")
    console.log("running",cmd)
    child_process.exec(cmd,(error, stdout, stderr) => {
        console.log("exec")
    })
}

run_qemu().then(()=>console.log("done"))
