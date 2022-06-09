import https from "https"
import os from "os"
import assert from "assert"
import * as child_process from 'child_process'
import util from "util"
import fs from 'fs'
import {
    IMAGE, IMAGE_FILE,
    IMAGE_URL,
    KERNEL_FILE,
    KERNEL_URL,
    PTB_FILE,
    PTB_URL,
    TMP_DIR
} from './common.js'

const execp = util.promisify(child_process.exec)

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
    console.log("running",s)
    let res = await execp(s)
    console.log("output is",res)
    return res
}

async function fetch_to_file_curl(KERNEL_URL, KERNEL_FILE) {
    console.log("curl to",KERNEL_URL, KERNEL_FILE)
    await execp(`curl -sSL "${KERNEL_URL}" -o "${KERNEL_FILE}"`)
    console.log("done downloading")
}

async function fetch_images() {
    // await fetch_to_file_curl(KERNEL_URL,KERNEL_FILE)
    // await fetch_to_file_curl(PTB_URL, PTB_FILE)
    // await fetch_to_file_curl(IMAGE_URL, IMAGE_FILE)
    // console.log("unziping")
    // let cwd = process.cwd()
    // process.chdir(TMP_DIR)
    await exec_command('git clone https://github.com/dhruvvyas90/qemu-rpi-kernel')
    // await exec_command(`unzip -d ${TMP_DIR} ${IMAGE_FILE}`)
    // console.log("done unzipping")
    // process.chdir(cwd)
}

if(os.type() === 'Darwin') {
    // assert(check_command_exists("brew &> /dev/null"),'Homebrew is missing. please install it')
    // install_qemu()
    fs.mkdirSync(TMP_DIR, {recursive:true})
    fetch_images().then(()=>{
        console.log("done with images")
    })
}
