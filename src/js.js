/* import("/pkg/genetic_algorithm_bg.js").then(wasm => {
    log(wasm)
    wasm.init();
}) */

import * as wasm from "/pkg/genetic_algorithm.js"
await wasm.default()

const WORLD_SETTINGS = {
    wolf_count: 4,
    sheep_count: 40,
    size: 1024
}

class App {
    constructor() {

        this.renderer = new Renderer.default(WORLD_SETTINGS.sheep_count, WORLD_SETTINGS.wolf_count)
        this.world = new wasm.World(WORLD_SETTINGS.sheep_count, WORLD_SETTINGS.wolf_count, WORLD_SETTINGS.size)
        log(`Simulation world started with seed [${this.world.seed}].`)
        log(this.world.get_quadtree())


        this.initDebugCanvas()
        this.initListeners()

        /* setInterval(this.update.bind(this), 2000) */
        this.update()
    }

    initListeners() {
        window.addEventListener("resize", () => {
            this.renderer.setSize(innerWidth, innerHeight)
        })

        if (this.canvas) {
            this.mouse = {
                x: 0,
                y: 0
            }
            this.canvas.addEventListener("mousemove", e => {
                this.mouse.x = e.offsetX;
                this.mouse.y = e.offsetY;
                /* log(this.mouse) */
                this.update()
            })
        }
    }

    initDebugCanvas() {
        this.canvas = document.createElement("canvas");
        this.ctx = this.canvas.getContext("2d");
        document.body.appendChild(this.canvas);
        this.canvas.width = 1024;
        this.canvas.height = 1024;
        this.canvas.id = "debugCanvas"
    }

    update() {
        /* requestAnimationFrame(this.update.bind(this)) */
        /* log("update") */


        if (this.canvas) {
            let active_quad = this.world.activate(this.mouse.x, this.mouse.y)
            log(active_quad)
            active_quad = active_quad == null ? "none" : active_quad
            let then = performance.now();
            let q = this.world.get_quadtree()
            /* log(q); */

            // Draw Quads
            if (true) {
                this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)
                let i = 0;
                q.forEach(quad => {
                    /* log(quad) */
                    let color = hexPalette[i % hexPalette.length];
                    if (quad.name == active_quad) {
                        color = "#ff00ff"
                        console.log(active_quad)
                    } else {
                        /* log(quad) */
                    }
                    const rgb = transparent_hex(color, quad.level / 10);
                    /* log(color) */
                    this.ctx.lineWidth = 5;
                    this.ctx.strokeStyle = color;
                    this.ctx.fillStyle = rgb
                    this.ctx.beginPath();
                    this.ctx.rect(quad.position[0], quad.position[1], quad.size, quad.size);
                    this.ctx.fill()
                    this.ctx.stroke()
                    this.ctx.closePath();


                    this.ctx.font = "12pt sans-serif"
                    this.ctx.fillStyle = "black"
                    this.ctx.fillText(`${quad.name}@${quad.position[0]},${quad.position[1]}`, quad.position[0], quad.position[1] + quad.size / 2);

                    i++;
                })
            }

            // Draw Agents
            if (true) {
                const agents = this.world.get_agents();
                /* log(agents) */
                for (let i = 0; i < agents.positions.length; i++) {
                    /* log(agents.positions[i], agents.types[i]) */
                    switch (agents.types[i]) {
                        case 0: // Wolf
                            this.ctx.fillStyle = "red"
                            break;
                        case 1: // Sheep
                            this.ctx.fillStyle = "white"
                            break;
                        case 2: // Grass
                            this.ctx.fillStyle = "green"
                            break;
                    }
                    this.ctx.beginPath();
                    this.ctx.arc(agents.positions[i][0], agents.positions[i][1], 10, 0, Math.PI * 2);
                    this.ctx.fill();
                    this.ctx.closePath()
                    /* log(`Drew ${i} at ${agents.positions[i][0]}/${agents.positions[i][1]}`) */
                }
            }

            // Draw Tree
            if (true) {
                /* this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height); */
                /* log(q) */
                let levels = []
                let relationships = {}
                q.forEach(quad => {
                    if (levels[quad.level]) {
                        levels[quad.level].push(quad)
                    } else {
                        levels[quad.level] = [quad]
                    }
                })

                // Figure out node relationships
                const then = performance.now();
                for (let i = 0; i < levels.length; i++) {
                    if (i > 0) {
                        let parent;
                        levels[i].forEach(node => {
                            levels[i - 1].forEach(potential_parent => {
                                potential_parent.child_nodes.forEach(child_node => {
                                    if (child_node.name == node.name) {
                                        parent = potential_parent.name
                                        /* log(child_node.name, node.name, potential_parent.name) */
                                        relationships[child_node.name] = parent
                                    }
                                })
                            })
                        })
                    }
                }
                log(relationships)
                log(`Relationships for ${q.length} nodes computed in ${performance.now() - then}ms.`)

                /* log(levels) */
                log(q)
                let _y = 60;
                const side = 40;
                this.ctx.font = "13pt sans-serif"

                let named_node_positions = {}
                // Draw the levels of the QuadTree
                levels.forEach(level => {
                    /* log(level) */
                    for (let i = -level.length / 2; i < level.length / 2; i++) {
                        const index = (i + level.length / 2)
                        /* log(level[index]) */

                        const x = this.canvas.width / 2 + i * 50 - side / 2
                        const y = _y - side / 2;

                        named_node_positions[level[index].name] = [x, y]

                        this.ctx.fillStyle = "black"
                        this.ctx.beginPath();
                        this.ctx.fillRect(x, y, side, side)
                        /* this.ctx.arc(this.canvas.width / 2 + i * 50, y, 2, 0, Math.PI * 2) */
                        this.ctx.fill()
                        this.ctx.closePath()


                        const parent_name = relationships[level[index].name];
                        const parent = named_node_positions[parent_name];
                        if (parent) {
                            /* log(parent_name, parent) */
                            this.ctx.strokeStyle = "black"
                            this.ctx.lineWidth = 1
                            this.ctx.beginPath();
                            this.ctx.moveTo(x, y);
                            this.ctx.lineTo(parent[0] + side / 2, parent[1] + side)
                            this.ctx.stroke()
                            this.ctx.closePath()
                        }


                        this.ctx.translate(x + 10, y + 10)
                        this.ctx.rotate(Math.PI / 3);
                        this.ctx.fillStyle = "red"
                        this.ctx.fillText(level[index].name, 0, 0)

                        this.ctx.resetTransform();
                    }
                    _y += 100;
                })
            }
            /* log(`Quadtree fetch & draw took ${performance.now() - then} ms`) */
        }

        /* this.renderer.render(); */
    }
}

const hexPalette = [
    "#003f5c",
    "#2f4b7c",
    "#665191",
    "#a05195",
    "#d45087",
    "#f95d6a",
    "#ff7c43",
    "#ffa600"
]

function hexToRgb(hex) { // Thanks to Tim Down @ https://stackoverflow.com/a/5624139
    var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : null;
}

function transparent_hex(hex, alpha) {
    if (alpha > 1 || alpha < 0) console.error('Alpha must be normalized (is currently ${alpha})');
    const rgb = hexToRgb(hex);
    return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`
}

document.readyState == "complete" ? window.app = new App() :
    addEventListener("load", () => {
        window.app = new App()
        log("App started")
    })