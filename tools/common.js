//https://downloads.raspberrypi.org/raspios_lite_armhf/images/raspios_lite_armhf-2021-01-12/2021-01-11-raspios-buster-armhf-lite.zip

export const IMAGE='2021-10-30-raspios-bullseye-armhf-lite'
// export const KERNEL='kernel-qemu-5.10.63-bullseye'
export const KERNEL='kernel-qemu-4.19.50-buster'
// export const PTB='versatile-pb-bullseye-5.10.63.dtb'
export const PTB='versatile-pb-buster.dtb'
// const TMP_DIR=`${process.env['HOME']}/qemu_vms`
export const TMP_DIR=`qemu_vms`
export const KERNEL_FILE=`${TMP_DIR}/${KERNEL}`
export const PTB_FILE=`${TMP_DIR}/${PTB}`
export const IMAGE_FILE = `${TMP_DIR}/${IMAGE}.zip`

//# commit hash to use for the https://github.com/dhruvvyas90/qemu-rpi-kernel/ repo:
// export const  COMMIT_HASH='061a3853cf2e2390046d163d90181bde1c4cd78f'
export const  COMMIT_HASH='86571772bb993c28bf8b925c202bc66dc63bcd45'
export const IMAGE_URL=`https://downloads.raspberrypi.org/raspios_lite_armhf/images/raspios_lite_armhf-2021-11-08/${IMAGE}.zip`
//                      https://downloads.raspberrypi.org/raspios_lite_armhf/images/raspios_lite_armhf-2021-11-08/2021-10-30-raspios-bullseye-armhf-lite.zip
export const KERNEL_URL=`https://github.com/dhruvvyas90/qemu-rpi-kernel/blob/${COMMIT_HASH}/${KERNEL}?raw=true`
export const PTB_URL=`https://github.com/dhruvvyas90/qemu-rpi-kernel/blob/${COMMIT_HASH}/${PTB}?raw=true`
