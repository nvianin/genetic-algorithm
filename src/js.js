/* import("/pkg/genetic_algorithm_bg.js").then(wasm => {
    log(wasm)
    wasm.init();
}) */

import * as wasm from "/pkg/genetic_algorithm.js"
await wasm.default()

const WORLD_SETTINGS = {
    wolf_count: 1,
    sheep_count: 10,
    size: 1024
}

class App {
    constructor() {

        this.renderer = new Renderer.default(WORLD_SETTINGS.sheep_count, WORLD_SETTINGS.wolf_count)
        this.world = new wasm.World(WORLD_SETTINGS.sheep_count, WORLD_SETTINGS.wolf_count, WORLD_SETTINGS.size)
        log(`Simulation world started with seed [${this.world.seed}].`)
        log(this.world.get_quadtree())

        this.initListeners()

        this.initDebugCanvas()

        this.update()
    }

    initListeners() {
        window.addEventListener("resize", () => {
            this.renderer.setSize(innerWidth, innerHeight)
        })
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
        requestAnimationFrame(this.update.bind(this))
        log("update")

        if (this.canvas) {
            this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)
            /* let then = performance.now(); */
            let q = this.world.get_quadtree()
            for (let i = 0; i < q.locations.length; i++) {
                if (!q.has_child_nodes[i]) {
                    const color = hexPalette[i % hexPalette.length];
                    this.ctx.strokeStyle = color
                    this.ctx.lineWidth = 5;
                    this.ctx.fillStyle = "rgba(255,0,0,.1)"
                    this.ctx.beginPath();
                    this.ctx.rect(q.locations[i][0], q.locations[i][1], q.sizes[i], q.sizes[i]);
                    /* this.ctx.fill() */
                    this.ctx.stroke()
                    this.ctx.closePath();
                    for (let j = 0; j < q.children[i].length; j++) {
                        switch (q.children_type[i][j]) {
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
                        this.ctx.arc(q.children[i][j][0], q.children[i][j][1], .01, 0, Math.PI * 2);
                        this.ctx.fill();
                    }
                }
            }
            /* log(`Quadtree fetch & draw took ${performance.now() - then} ms`) */
        }

        this.renderer.render();
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

document.readyState == "complete" ? window.app = new App() :
    addEventListener("load", () => {
        window.app = new App()
        log("App started")
    })