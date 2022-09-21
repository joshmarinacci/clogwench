import {make_logger} from "josh_js_util"
import {file_readable, getFiles} from "josh_node_util"
import {promises as fs} from 'fs'
import path from 'path'
import toml from "toml"


const log = make_logger("BUILD")

async function add_to_graph(dir, graph) {
    let cargo_file_name = path.join(dir, 'Cargo.toml')
    let cargo_buffer = await fs.readFile(cargo_file_name)
    let data = toml.parse(cargo_buffer.toString());
    Object.keys(data.dependencies).forEach(key => {
        let val = data.dependencies[key]
        if(val['path']) {
            let clean = path.basename(val.path.trim())
            let name = path.basename(dir)
            console.log("link",clean,"-->",name)
            if(!graph.has(clean)) {
                graph.set(clean, [])
            }
            graph.get(clean).push(name)
            // graph[clean] = path.basename(dir)
        }
    })
}

const l = (...args) => log.info(...args)


async function doit() {
    let base = '../..'
    let cargos = []
    await getFiles(base,async (filepath) => {
        if(filepath.includes('node_modules')) return
        if(filepath.includes('.git')) return
        if(filepath.includes('/vm/')) return
        if(filepath.includes('/target/')) return
        if(filepath.endsWith('Cargo.toml')) {
            let crate_dir = path.dirname(filepath)
            if("../.." === crate_dir) return
            if(".." === crate_dir) return
            console.log("crate dir is",crate_dir)
            cargos.push(crate_dir)
        }
    })
    console.log("checking the dirs")
    let graph = new Map()

    for(let dir of cargos) {
        if (await file_readable(path.join(dir,'Cargo.toml'))) {
            await add_to_graph(dir,graph)
        }
    }
    log.info("final graph",graph)
    let out = ""
    out += "graph TD\n"
    for (let name of graph.keys()) {
        // console.log("key is ",name);
        for(let dep of graph.get(name)) {
            // console.log("dep is ",dep)
            out += `   ${name} --> ${dep}\n`
        }
    }
    // console.log(`graph TD\n`+out.join("\n"))
    console.log(out)

    out = `
# architecture diagram
        
\`\`\`mermaid
${out}
\`\`\`
# thats it?        
    `
    await fs.writeFile("diagram.md",out)

}

doit().then(()=>console.log("done")).catch((e)=>console.log("error",e))


/*
- [ ] Make dep map
- [ ] Find all cargo files
- [ ] Parse with a toml parser
- [ ] Find all deps
- [ ] If dep is a local path add it to the dep map
- [ ] Generate mermaid doc

 */


