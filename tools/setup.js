import https from "https"
import os from "os"
import assert from "assert"
import * as child_process from 'child_process'
import util from "util"
import fs from 'fs'

const execp = util.promisify(child_process.exec)

const IMAGE='2020-02-13-raspbian-buster-lite'
const KERNEL='kernel-qemu-5.4.51-buster'
const PTB='versatile-pb-buster-5.4.51.dtb'

// const TMP_DIR=`${process.env['HOME']}/qemu_vms`
const TMP_DIR=`qemu_vms`
const KERNEL_FILE=`${TMP_DIR}/${KERNEL}`
const PTB_FILE=`${TMP_DIR}/${PTB}`

//# commit hash to use for the https://github.com/dhruvvyas90/qemu-rpi-kernel/ repo:
const  COMMIT_HASH='061a3853cf2e2390046d163d90181bde1c4cd78f'

const IMAGE_URL=`https://downloads.raspberrypi.org/raspbian_lite/images/raspbian_lite-2020-02-14/${IMAGE}.zip`
const KERNEL_URL="https://github.com/dhruvvyas90/qemu-rpi-kernel/blob/${COMMIT_HASH}/${KERNEL}?raw=true"
const PTB_URL="https://github.com/dhruvvyas90/qemu-rpi-kernel/blob/${COMMIT_HASH}/${PTB}?raw=true"

assert.equal(os.type(),"Darwin","we only support MacOS currently")

function check_command_exists(cmd) {
    try {
        let ch = child_process.execSync(cmd, { stdio:['ignore'], stderr:['ignore']})
        // console.log("result is",ch.toString())
        return true
    } catch (e) {
        console.log("error")
        return false
    }
}

function install_qemu() {
    console.log("installing qemu")
    child_process.execSync("brew install qemu")
}

function fetch_to_file(src_url, dst_file) {
    console.log("downloading",src_url)
    console.log("to",dst_file)
    let file = fs.createWriteStream(dst_file)
    return new Promise((res,rej)=>{
        let respsent = false
        https.get(src_url, resp => {
            console.log('status code',resp.statusCode)
            console.log("headers",resp.headers)
            let len = parseInt(resp.headers['content-length'])
            let progress = 0
            console.log("length",len)
            resp.on('data',(d)=>{
                progress += d.length
                console.log('progress',(progress/len).toFixed(3))
            })
            resp.pipe(file)
            file.on('finish',() => {
                file.close(()=>{
                    if(respsent) return
                    respsent = true
                    res()
                })
            })
        }).on('error',err => {
            if(respsent) return;
            respsent = true
            rej(err)
        })
    })
}

async function exec_command(s) {
    let res = await execp(s,{ stdio:['ignore'], stderr:['ignore']})
    // console.log("output is",res)
    return res
}

async function fetch_images() {
    // await fetch_to_file(KERNEL_URL,KERNEL_FILE)
    // await fetch_to_file(PTB_URL, PTB_FILE)
    await fetch_to_file(IMAGE_URL, `${IMAGE}.zip`)
    // await exec_command(`unzip ${IMAGE}.zip`)
    // await exec_command(`${IMAGE}.zip`)
}

if(os.type() === 'Darwin') {
    // assert(check_command_exists("brew &> /dev/null"),'Homebrew is missing. please install it')
    // install_qemu()
    fs.mkdirSync(TMP_DIR, {recursive:true})
    process.chdir(TMP_DIR)
    fetch_images().then(()=>{
        console.log("done with images")
    })
}
