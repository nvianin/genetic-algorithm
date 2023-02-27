/* import("/pkg/genetic_algorithm_bg.js").then(wasm => {
    log(wasm)
    wasm.init();
}) */

import * as wasm from "/pkg/genetic_algorithm.js"
await wasm.default()

const WORLD_SETTINGS = {
    wolf_count: 32,
    sheep_count: 128,
    size: 1024
}

const STATE_COLOURS = {
    0: "#eeeeee", // Idle
    1: "red", // Hunting
    2: "yellow", // Fleeing
    3: "green", // Eating
    4: "grey", // Dead
}

class App {
    constructor() {

        this.renderer = new Renderer.default(WORLD_SETTINGS.sheep_count, WORLD_SETTINGS.wolf_count, WORLD_SETTINGS.size)
        this.world = new wasm.World(WORLD_SETTINGS.sheep_count, WORLD_SETTINGS.wolf_count, WORLD_SETTINGS.size)
        log(`Simulation world started with seed [${this.world.seed}].`);
        /* log(this.world.get_quadtree()); */

        this.continue_render = true

        performance = performance ? performance : Date;
        this.start_time = performance.now();

        this.initInterface()
        this.initDebugCanvas()
        this.initListeners()

        /* setInterval(this.update.bind(this), 2000) */
        this.update()
    }

    initListeners() {
        window.addEventListener("resize", () => {
            this.renderer.setSize(innerWidth, innerHeight)
        })

        this.queryMethod = 0; // 0 = quadtree, 1 = brute force

        this.mousepicked_agent = null
        this.hovered_agent = null;
        this.prev_mousepicked_agent = null;

        this.renderer.renderer.domElement.addEventListener("mousemove", e => {
            this.renderer.three_mouse.x = e.clientX / innerWidth * 2 - 1;
            this.renderer.three_mouse.y = -(e.clientY / innerHeight) * 2 + 1;

            this.renderer.mousecaster.setFromCamera(this.renderer.three_mouse, this.renderer.camera);
            const intersects = this.renderer.mousecaster.intersectObject(this.renderer.ground, false);
            if (intersects.length > 0) {
                this.renderer.mouse.x = intersects[0].point.x + WORLD_SETTINGS.size / 2;
                this.renderer.mouse.y = intersects[0].point.z + WORLD_SETTINGS.size / 2;

                const nearby_agents = this.world.get_agents_in_radius(this.renderer.mouse.x, this.renderer.mouse.y, 10)
                if (nearby_agents.positions.length > 0) {
                    this.hovered_agent = nearby_agents.ids[0];
                    /* if (this.renderer.tracking_agent && this.prev_mousepicked_agent != this.mousepicked_agent.ids[0]) {
                        this.renderer.tracking_agent = false;
                    } */
                    /* this.prev_mousepicked_agent = this.mousepicked_agent.ids[0]
                    log(this.mousepicked_agent) */
                } else {
                    this.hovered_agent = null;
                }
            }
        })

        this.renderer.tracking_agent = false;
        this.renderer.renderer.domElement.addEventListener("mousedown", e => {
            if (this.renderer.tracking_agent && !this.hovered_agent) {
                this.renderer.tracking_agent = false;
                return;
            }
            if (this.hovered_agent) {
                this.renderer.tracking_agent = true;
                this.mousepicked_agent = this.hovered_agent;

                this.renderer.fake_cam.copy(this.renderer.camera)
                log(this.hovered_agent)
                const selected = this.inspected_agents.find(e => {
                    return e.uuid == this.hovered_agent
                })
                this.agent_inspector.scrollTo(0, selected.offsetTop - 100)
                selected.click()

                /* this.renderer.controller.enabled = !this.renderer.tracking_agent; */
            }
        })

        this.renderer.camera.userData.scroll = 0;
        this.renderer.renderer.domElement.addEventListener("wheel", e => {
            if (this.renderer.tracking_agent) {
                if (e.deltaY < 0 && this.renderer.camera.position.y <= 50 || e.deltaY > 0 && this.renderer.camera.position.y >= 600) {
                    return;
                }
                this.renderer.camera.userData.scroll += e.deltaY * .5;
            }
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
                /* this.update() */
            })
            this.canvas.addEventListener("mousedown", e => {
                this.queryMethod = !this.queryMethod
                log(this.queryMethod)
            })
        }

        window.addEventListener("keydown", e => {
            switch (e.key) {
                case " ": // Space
                    this.continue_render = !this.continue_render
                    log(this.continue_render ? "Continuing render." : "Pausing render.");
                    break;
            }
        })
    }

    initInterface() {
        this.agent_inspector = document.createElement("div");
        this.agent_inspector.id = "agent_inspector";
        document.body.appendChild(this.agent_inspector);

        this.inspected_agents = []

        this.agent_element_template = document.createElement("button")
        this.agent_element_template.className = `agent_element`
        this.agent_element_template.innerText = `Agent`
        this.agent_element_template.uuid = "lol";

        this.agent_info_template = document.createElement("span");
        this.agent_info_template.className = "agent_info";

        this.agent_inspector.onclick = e => {
            if (!e.target.child) return
            log(e.target.child)
            e.target.classList.toggle("active")
            e.target.child.style.display = e.target.classList.contains("active") ? "flex" : "none"
            this.mousepicked_agent = e.target.uuid
            /* log(this.mousepicked_agent) */
            this.renderer.tracking_agent = true;
        }
    }

    refreshInterface(agents) {
        /* if (agents.positions.length == this.inspected_agents.length) return; */

        if (agents.positions.length < this.inspected_agents.length) {
            while (agents.positions.length != this.inspected_agents.length) {
                const to_remove = this.inspected_agents.pop();
                this.agent_inspector.removeChild(to_remove);
            }
        } else {
            while (agents.positions.length != this.inspected_agents.length) {
                const agent_element = this.agent_element_template.cloneNode(false);
                const agent_info = this.agent_info_template.cloneNode(false);
                agent_info.style.display = "none"
                agent_element.child = agent_info;
                const i = this.inspected_agents.push(agent_element) - 1;
                this.agent_inspector.appendChild(agent_element);
                this.agent_inspector.appendChild(agent_info)

                let type = "none"
                let imgsrc = "none"
                switch (agents.types[i]) {
                    case 0:
                        type = "Wolf"
                        imgsrc = "./rsc/textures/wolf_icon.png"
                        break;
                    case 1:
                        type = "Sheep"
                        imgsrc = "./rsc/textures/sheep_icon.png"
                        break;
                    case 2:
                        type = "Grass"
                        imgsrc = "./rsc/textures/grass_icon.png"
                        break;
                }

                agent_element.innerText = type;
                agent_element.uuid = agents.ids[i]
                const img = document.createElement("img");
                img.src = imgsrc;
                if (imgsrc == "none") log(agents.types[i])
                agent_info.appendChild(img)


                const healthbar = document.createElement("span");
                healthbar.innerText = "Health"
                healthbar.classList.add("healthbar", "stat_bar")
                agent_info.healthbar = healthbar
                agent_info.appendChild(healthbar)
                if (agents.types[i] != 2) {
                    const hungerbar = document.createElement("span");
                    hungerbar.innerText = "Hunger"
                    hungerbar.classList.add("hungerbar", "stat_bar")
                    agent_info.hungerbar = hungerbar
                    agent_info.appendChild(hungerbar)

                }

                const agent_info_text = document.createElement("span")
                agent_info_text.classList.add("agent_info_text")
                agent_info.appendChild(agent_info_text);
                agent_element.text = agent_info_text;
            }
        }

        for (let i = 0; i < agents.positions.length; i++) {
            if (this.inspected_agents[i].innerText == "Grass") {
                this.inspected_agents[i].child.healthbar.style.width = `${agents.vitals[i][0] * .9}%`
                /* if(agents.vitals[i][0] != 100){
                    log(agents.vitals[i][0])
                } */
                continue
            };
            let col = STATE_COLOURS[agents.states[i]]

            this.inspected_agents[i].style.boxShadow = `inset 0 0 12px 3px ${col}`
            /* log(col) */

            if (this.inspected_agents[i].child.style.display == "none") continue;
            let state_name = "Idle"
            switch (agents.states[i]) {
                case 1:
                    state_name = "Hunting"
                    break;
                case 2:
                    state_name = "Fleeing"
                    break
                case 3:
                    state_name = "Reproducing"
                    break;
                case 4:
                    state_name = "Dead"
                    break;
            }
            this.inspected_agents[i].child.healthbar.style.width = `${agents.vitals[i][0] * .9}%`
            this.inspected_agents[i].child.hungerbar.style.width = `${agents.vitals[i][1] * .9}%`
            /* log(agents.vitals[i][1]) */
            this.inspected_agents[i].text.innerText =
                `
                State: ${state_name}
                Position: ${Math.floor(agents.positions[i][0])},${Math.floor(agents.positions[i][1])}
                Genotype:
                Body size: ${Math.floor(agents.genotypes[i][0] * 100) / 100} 
                Sight: ${Math.floor(agents.genotypes[i][1] * 100) / 100}
                Muscle mass: ${Math.floor(agents.genotypes[i][2] * 100) / 100}
                ---
                Hunger rate: ${Math.floor(agents.genotypes[i][3] * 100) / 100}
                Health scale: ${Math.floor(agents.genotypes[i][4] * 100) / 100}
                Speed: ${Math.floor(agents.genotypes[i][5] * 100) / 100}
                Gestation time: ${Math.floor(agents.genotypes[i][6] * 100) / 100}


            `
        }
    }

    initDebugCanvas() {
        this.benchmarking_data = {
            quadtree: [],
            brute_force: []
        }

        this.canvas = document.createElement("canvas");
        this.ctx = this.canvas.getContext("2d");
        document.body.appendChild(this.canvas);
        this.canvas.width = 1024;
        this.canvas.height = 1024;
        this.canvas.id = "debugCanvas"
    }

    update() {
        requestAnimationFrame(this.update.bind(this))
        this.time = (performance.now() - this.start_time) / 1000;
        if (!this.continue_render) {
            /* log(`Step took ${performance.now() - then}ms.`); */
            return
        }
        const then = performance.now();
        this.world.step(true, this.time);
        /* log("update") */

        const agents = this.world.get_agents();

        /* log(agents.positions.length) */
        this.refreshInterface(agents);
        this.renderer.render(agents);

        if (this.hovered_agent && this.renderer.selection_circle) {
            const index = agents.ids.indexOf(this.hovered_agent);
            /* log(index)
            log(this.mousepicked_agent.ids, agents.ids) */
            this.renderer.selection_circle.position.x = agents.positions[index][0] - WORLD_SETTINGS.size / 2
            this.renderer.selection_circle.position.z = agents.positions[index][1] - WORLD_SETTINGS.size / 2

            this.renderer.renderer.domElement.style.cursor = "pointer"
            this.renderer.selection_circle.material.color = STATE_COLOURS[agents.states[index]]

        } else {
            this.renderer.renderer.domElement.style.cursor = "default"
        }
        // Track a single agent
        if (this.renderer.tracking_agent && this.mousepicked_agent) {
            const index = agents.ids.indexOf(this.mousepicked_agent);

            this.renderer.camera.position.y += this.renderer.camera.userData.scroll;
            this.renderer.camera.userData.scroll = 0;

            this.renderer.selection_circle.material.color = STATE_COLOURS[agents.states[index]]
            this.renderer.selection_circle.position.x = agents.positions[index][0] - WORLD_SETTINGS.size / 2
            this.renderer.selection_circle.position.z = agents.positions[index][1] - WORLD_SETTINGS.size / 2

            /* this.renderer.camera.position.y = WORLD_SETTINGS.size / 2; */
            this.renderer.camera.position.x = agents.positions[index][0] - WORLD_SETTINGS.size / 2 + 100
            this.renderer.camera.position.z = agents.positions[index][1] - WORLD_SETTINGS.size / 2 - 100

            this.renderer.camera.lookAt(
                agents.positions[index][0] - WORLD_SETTINGS.size / 2,
                0,
                agents.positions[index][1] - WORLD_SETTINGS.size / 2
            )
        } else {
            // Use regular controls
            this.renderer.camera.position.copy(this.renderer.fake_cam.position);
            this.renderer.camera.rotation.copy(this.renderer.fake_cam.rotation);
        }

        if (this.canvas && false) {
            let active_quad = this.world.activate(this.mouse.x, this.mouse.y)
            /* if (active_quad) {
                log(active_quad.name, active_quad.position)
            } */
            active_quad.name = active_quad.name == null ? "none" : active_quad.name
            let then = performance.now();
            let q = this.world.get_quadtree()
            /* log(q); */

            this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)
            // Draw Quads
            if (false) {
                let i = 0;
                q.forEach(quad => {
                    /* log(quad) */
                    let color = hexPalette[i % hexPalette.length];
                    if (quad.name == active_quad.name) {
                        color = "#ff00ff"
                        /* console.log(active_quad) */
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

                    if (false) {
                        this.ctx.font = "12pt sans-serif"
                        this.ctx.fillStyle = "black"
                        this.ctx.fillText(`${quad.name}@${quad.position[0]},${quad.position[1]}+${quad.size}`, quad.position[0], quad.position[1] + quad.size / 2);
                    }
                    i++;
                })
            }

            // Draw Agents
            if (false) {
                /* log(agents) */

                let then = performance.now();
                let nearby_points;
                if (this.queryMethod && false) {
                    nearby_points = {
                        ids: [],
                        positions: [],
                        types: []
                    }
                    for (let i = 0; i < agents.positions.length; i++) {
                        /* let then = performance.now();
                        let condition = Math.sqrt(Math.pow(p[0] - this.mouse.x, 2) + Math.pow(p[1] - this.mouse.y, 2)) < 100
                        log(`Brute force sqrt took ${performance.now() - then}ms with result ${condition}.`);

                        then = performance.now();
                        condition = Math.pow(p[0] - this.mouse.x, 2) + Math.pow(p[1] - this.mouse.y, 2) < Math.pow(100, 2)
                        log(`Brute force pow took ${performance.now() - then}ms with result ${condition}.`); */

                        if (Math.pow(agents.positions[i][0] - this.mouse.x, 2) +
                            Math.pow(agents.positions[i][1] - this.mouse.y, 2) <
                            Math.pow(50, 2)) {
                            nearby_points.positions.push(agents.positions[i])
                            nearby_points.ids.push(agents.ids[i])
                            nearby_points.types.push(agents.types[i])
                        }
                    }
                    /* log(nearby_points) */
                } else {
                    nearby_points = this.world.get_agents_in_radius(this.mouse.x, this.mouse.y, 50);
                }
                if (this.queryMethod) {
                    /* log(`Brute force took ${performance.now() - then}ms.`) */
                    this.benchmarking_data.brute_force.push(performance.now() - then);
                } else {
                    /* log(`Quadtree took ${performance.now() - then}ms.`) */
                    this.benchmarking_data.quadtree.push(performance.now() - then);
                }
                if (nearby_points.positions.length > 0 && this.continue_render) {
                    /* log(nearby_points) */
                } else {
                    /* log("No nearby points.") */
                }

                // log the average time taken for each query method
                /* if (this.benchmarking_data.quadtree.length > 100) {
                    let sum = 0;
                    for (let i = 0; i < this.benchmarking_data.quadtree.length; i++) {
                        sum += this.benchmarking_data.quadtree[i];
                    }
                    log(`Average time taken for quadtree: ${sum / this.benchmarking_data.quadtree.length}ms.`)
                }
                if (this.benchmarking_data.brute_force.length > 100) {
                    let sum = 0;
                    for (let i = 0; i < this.benchmarking_data.brute_force.length; i++) {
                        sum += this.benchmarking_data.brute_force[i];
                    }
                    log(`Average time taken for brute force: ${sum / this.benchmarking_data.brute_force.length}ms.`)
                } */


                for (let i = 0; i < agents.positions.length; i++) {
                    /* log(agents.positions[i], agents.types[i]) */
                    let radius = 2;
                    if (nearby_points.ids.includes(agents.ids[i])) {
                        this.ctx.fillStyle = "#ff00ff"
                        radius = 10;
                        /* log(i) */
                    } else {
                        switch (agents.types[i]) {
                            case 0: // Wolf
                                this.ctx.fillStyle = "red"
                                break;
                            case 1: // Sheep
                                this.ctx.fillStyle = "#aaaaaa"
                                break;
                            case 2: // Grass
                                this.ctx.fillStyle = "green"
                                break;
                        }
                    }

                    this.ctx.beginPath();
                    this.ctx.arc(agents.positions[i][0], agents.positions[i][1], radius, 0, Math.PI * 2);
                    this.ctx.fill();
                    this.ctx.closePath()
                    /* log(`Drew ${i} at ${agents.positions[i][0]}/${agents.positions[i][1]}`) */
                }

                // Draw Tree
                if (false) {
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
                    /* log(relationships) */
                    /* log(`Relationships for ${q.length} nodes computed in ${performance.now() - then}ms.`) */

                    /* log(levels) */
                    /* log(q) */
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

                            this.ctx.fillStyle = level[index].name == active_quad.name ? "green" : "black"
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
                /* log(`
                        Quadtree fetch & draw took $ {
                            performance.now() - then
                        }
                        ms `) */
            }

            /* this.renderer.render(); */
        }
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
    return `
                    rgba(${rgb.r
        }, ${rgb.g
        }, ${rgb.b
        }, ${alpha
        })
                    `
}

document.readyState == "complete" ? window.app = new App() :
    addEventListener("load", () => {
        window.app = new App()
        log("App started")
    })