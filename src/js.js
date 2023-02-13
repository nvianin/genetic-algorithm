/* import("/pkg/genetic_algorithm_bg.js").then(wasm => {
    log(wasm)
    wasm.init();
}) */

import * as wasm from "/pkg/genetic_algorithm.js"
await wasm.default()

const WORLD_SETTINGS = {
    wolf_count: 2,
    sheep_count: 8,
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

        if (this.canvas) {
            this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)
            let q = this.world.get_quadtree()
            for (let i = 0; i < q.locations.length; i++) {
                this.ctx.fillStyle = "red"
                this.ctx.fillRect(q.locations[i][0], q.locations[i][1], q.sizes[i], q.sizes[i]);
            }
        }

        this.renderer.render();
    }
}

document.readyState == "complete" ? window.app = new App() :
    addEventListener("load", () => {
        window.app = new App()
        log("App started")
    })