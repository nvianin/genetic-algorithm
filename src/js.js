/* import("/pkg/genetic_algorithm_bg.js").then(wasm => {
    log(wasm)
    wasm.init();
}) */

import * as wasm from "/pkg/genetic_algorithm.js"
await wasm.default()

const WORLD_SETTINGS = {
    wolf_count: 16,
    sheep_count: 128,
    size: 1024
}

const STATE_COLOURS = {
    0: "#eeeeee", // Idle
    1: "red", // Hunting
    2: "yellow", // Fleeing
    3: "green", // Reproducing
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

        this.logging_data = {
            time: [],
            sheep: {
                count: [],
                body_size: [],
                sight_distance: [],
                muscle_mass: [],
                hunger_rate: [],
                health_scale: [],
                movement_speed: [],
                gestation_duration: [],
                health: [],
                hunger: [],
            },
            wolves: {
                count: [],
                body_size: [],
                sight_distance: [],
                muscle_mass: [],
                hunger_rate: [],
                health_scale: [],
                movement_speed: [],
                gestation_duration: [],
                health: [],
                hunger: [],
            },
            grass: {
                count: [],
                health: []
            },
        }
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
            if (this.mouse_down) {
                this.mouse_moved_while_down = true;
            }
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

        this.mouse_down = false;
        this.mouse_moved_while_down = false;

        this.renderer.renderer.domElement.addEventListener("mousedown", e => {
            this.mouse_down = true;
            this.mouse_moved_while_down = false;
        })

        this.renderer.tracking_agent = false;
        this.renderer.renderer.domElement.addEventListener("mouseup", e => {
            this.mouse_down = false;
            if (this.renderer.tracking_agent && !this.hovered_agent) {
                this.renderer.tracking_agent = false;
                return;
            }
            if (this.hovered_agent && !this.mouse_moved_while_down) {
                this.renderer.tracking_agent = true;
                this.mousepicked_agent = this.hovered_agent;

                this.renderer.fake_cam.copy(this.renderer.camera)
                log(this.hovered_agent)
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
        this.agent_inspector = document.getElementById("agent-inspector");
        this.agent_inspector_stats = document.getElementById("agent-stats");
        this.agent_inspector_title = document.getElementById("agent-title");
        this.agent_portrait = document.getElementById("agent-portrait")
        this.agent_health = document.getElementById("agent-health");
        this.agent_hunger = document.getElementById("agent-hunger");
        this.agent_defocus = document.getElementById("agent-defocus");

        this.agent_defocus.onclick = () => {
            this.renderer.tracking_agent = false;
            this.mousepicked_agent = null;
            this.hovered_agent = null;
        }

        this.agent_portrait.onclick = () => {
            this.renderer.tracking_agent = true
            this.mousepicked_agent = this.hovered_agent;
            log("Switching tracking mode by portrait click")
        }

        this.stats_drawer = document.getElementById("stats");
        this.stats_canvas = document.getElementById("stats-canvas");
        this.stats_canvas.ctx = this.stats_canvas.getContext("2d");
        this.stats_selector = document.querySelector("#stats-selector");
    }

    refreshInterface(agents) {
        if (!this.stats_drawer.classList.contains("stats-hidden")) {
            const selected_stat = this.stats_selector.value;
            const selected_category = this.stats_selector.options[this.stats_selector.selectedIndex].parentElement.getAttribute("value");

            this.stats_canvas.ctx.fillStyle = "white"
            this.stats_canvas.ctx.clearRect(0, 0, this.stats_canvas.width, this.stats_canvas.height);
            const width = this.stats_canvas.width;
            const height = this.stats_canvas.height;

            this.stats_canvas.ctx.strokeStyle = "red"
            this.stats_canvas.ctx.lineWidth = 2;
            this.stats_canvas.ctx.beginPath();
            this.stats_canvas.ctx.moveTo(0, this.logging_data[selected_category][selected_stat][0] * height);
            try {
                for (let i = 0; i < this.logging_data.time.length; i++) {
                    const x = (this.logging_data.time[i] - this.logging_data.time[0]) / (this.logging_data.time[this.logging_data.time.length - 1] - this.logging_data.time[0]) * width;
                    const y = this.logging_data[selected_category][selected_stat][i] / this.logging_data[selected_category].count[0] * height;
                    /* if (i % 100 == 0) { log(x, y, this.logging_data[selected_category]) } */
                    this.stats_canvas.ctx.fillStyle = "rgba(255, 255, 255, 0.5)";
                    this.stats_canvas.ctx.lineTo(x, height - y);
                }
            } catch (e) {
                console.error(e);
                log(selected_category, selected_stat);
            }
            this.stats_canvas.ctx.stroke()
            this.stats_canvas.ctx.closePath();
            // TODO: Calculate "max_seen" value for each stat and use it to scale the graph, now it normalizes according to the first value in the array
            this.stats_canvas.ctx.fillText(
                this.logging_data[selected_category][selected_stat][this.logging_data[selected_category][selected_stat].length - 1],
                width - 30,
                height - this.logging_data[selected_category][selected_stat][this.logging_data[selected_category][selected_stat].length - 1] / this.logging_data[selected_category].count[this.logging_data[selected_category].count.length - 1] * height + 10)
        }
    }

    updateInspector(index, agents) {
        let stateName = "Idle"
        switch (agents.states[index]) {
            case 1:
                stateName = "Hunting"
                break;
            case 2:
                stateName = "Fleeing"
                break;
            case 3:
                stateName = "Reproducing"
                break;
            case 4:
                stateName = "Dead"
                break;
        }
        let type = "Grass"
        let imgSrc = "./rsc/textures/grass_icon.png"
        switch (agents.types[index]) {
            case 0:
                type = "Wolf"
                imgSrc = "./rsc/textures/wolf_icon.png"
                break;
            case 1:
                type = "Sheep"
                imgSrc = "./rsc/textures/sheep_icon.png"
                break;
        }
        this.agent_inspector_title.innerHTML = type;
        this.agent_inspector_stats.innerText = `State: ${stateName}
            \nPosition: ${Math.floor(agents.positions[index][0])},${Math.floor(agents.positions[index][1])}
            \nHealth: ${Math.floor(agents.vitals[index][0])}
            `
        if (type != "Grass") {
            this.agent_inspector_stats.innerText +=
                `
            \nHunger: ${Math.floor(agents.vitals[index][1])}
            \nGenes
            \nBody size: ${agents.genotypes[index][0]}
            \nSight range: ${agents.genotypes[index][1]}
            \nMuscle mass: ${agents.genotypes[index][2]}
            \n
            \nHunger rate: ${agents.genotypes[index][3]}
            \nHealth scale: ${agents.genotypes[index][4]}
            \nMovement speed: ${agents.genotypes[index][5]}
            \nGestation time: ${agents.genotypes[index][6]}
            `
        }
        this.agent_portrait.src = imgSrc
        this.agent_health.style.width = `${agents.vitals[index][0] / 100 * 21}vh`
        this.agent_hunger.style.width = `${agents.vitals[index][1] / 100 * 21}vh`
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
        this.logData(agents);

        /* log(agents.positions.length) */
        this.refreshInterface(agents);
        this.renderer.render(agents);

        if (this.hovered_agent && this.renderer.selection_circle) {
            const index = agents.ids.indexOf(this.hovered_agent);
            if (index == -1) {
                // Handle case where the focused agent has died
                this.hovered_agent = null;
            }
            /* log(index)
            log(this.mousepicked_agent.ids, agents.ids) */
            this.renderer.selection_circle.position.x = agents.positions[index][0] - WORLD_SETTINGS.size / 2
            this.renderer.selection_circle.position.z = agents.positions[index][1] - WORLD_SETTINGS.size / 2

            this.renderer.renderer.domElement.style.cursor = "pointer"
            this.renderer.selection_circle.material.color = STATE_COLOURS[agents.states[index]]

            this.updateInspector(index, agents)
        } else {
            this.renderer.renderer.domElement.style.cursor = "default"
        }
        // Track a single agent
        if (this.renderer.tracking_agent && this.mousepicked_agent) {
            const index = agents.ids.indexOf(this.mousepicked_agent);
            if (index == -1) {
                // Handle case where the focused agent has died
                this.mousepicked_agent = null;
            }

            this.renderer.camera.position.y += this.renderer.camera.userData.scroll;
            this.renderer.camera.position.y = Math.max(0, Math.min(1000, this.renderer.camera.position.y));
            this.renderer.camera.userData.scroll = 0;

            this.renderer.selection_circle.material.color = STATE_COLOURS[agents.states[index]]
            this.renderer.selection_circle.position.x = agents.positions[index][0] - (WORLD_SETTINGS.size / 2)
            this.renderer.selection_circle.position.z = agents.positions[index][1] - WORLD_SETTINGS.size / 2

            /* this.renderer.camera.position.y = WORLD_SETTINGS.size / 2; */
            this.renderer.camera.position.x = agents.positions[index][0] - WORLD_SETTINGS.size / 2 + 100 * this.renderer.camera.position.y * .01
            this.renderer.camera.position.z = agents.positions[index][1] - WORLD_SETTINGS.size / 2 - 100 * this.renderer.camera.position.y * .01

            this.renderer.camera.lookAt(
                agents.positions[index][0] - WORLD_SETTINGS.size / 2,
                0,
                agents.positions[index][1] - WORLD_SETTINGS.size / 2
            )

            this.updateInspector(index, agents);

        } else {
            // Use regular controls
            this.renderer.camera.position.copy(this.renderer.fake_cam.position);
            this.renderer.camera.rotation.copy(this.renderer.fake_cam.rotation);
        }

        if (this.canvas && true) {
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
                        this.ctx.fillText(`${quad.name} @${quad.position[0]},${quad.position[1]} +${quad.size} `, quad.position[0], quad.position[1] + quad.size / 2);
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
                        log(`Brute force sqrt took ${ performance.now() - then }ms with result ${ condition }.`);

                        then = performance.now();
                        condition = Math.pow(p[0] - this.mouse.x, 2) + Math.pow(p[1] - this.mouse.y, 2) < Math.pow(100, 2)
                        log(`Brute force pow took ${ performance.now() - then }ms with result ${ condition }.`); */

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
                    /* log(`Brute force took ${ performance.now() - then } ms.`) */
                    this.benchmarking_data.brute_force.push(performance.now() - then);
                } else {
                    /* log(`Quadtree took ${ performance.now() - then } ms.`) */
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
                    log(`Average time taken for quadtree: ${ sum / this.benchmarking_data.quadtree.length } ms.`)
                }
                if (this.benchmarking_data.brute_force.length > 100) {
                    let sum = 0;
                    for (let i = 0; i < this.benchmarking_data.brute_force.length; i++) {
                        sum += this.benchmarking_data.brute_force[i];
                    }
                    log(`Average time taken for brute force: ${ sum / this.benchmarking_data.brute_force.length } ms.`)
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
                    /* log(`Drew ${ i } at ${ agents.positions[i][0] } /${agents.positions[i][1]}`) */
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

    logData(agents) {

        if (this.logging_data.time.length > 299) {
            const sub_indexes = ["wolves", "sheep", "grass"]
            sub_indexes.forEach(sub_index => {
                Object.keys(this.logging_data[sub_index]).forEach(key => {
                    this.logging_data[sub_index][key].shift()
                })
            })
            this.logging_data.time.shift()
        }

        let wolves = {
            body_size_tally: 0,
            sight_tally: 0,
            muscle_tally: 0,
            hunger_tally: 0,
            hunger_scale_tally: 0,
            health_tally: 0,
            health_scale_tally: 0,
            speed_tally: 0,
            gestation_tally: 0,

            count: 0
        }

        let sheep = {
            body_size_tally: 0,
            sight_tally: 0,
            muscle_tally: 0,
            hunger_tally: 0,
            hunger_scale_tally: 0,
            health_tally: 0,
            health_scale_tally: 0,
            speed_tally: 0,
            gestation_tally: 0,

            count: 0
        }

        let grass = {
            count: 0,
            health_tally: 0
        }

        for (let i = 0; i < agents.positions.length; i++) {

            switch (agents.types[i]) {
                case 0:
                    wolves.count++;
                    wolves.body_size_tally += agents.genotypes[i][0];
                    wolves.sight_tally += agents.genotypes[i][1];
                    wolves.muscle_tally += agents.genotypes[i][2];
                    wolves.hunger_scale_tally += agents.genotypes[i][3];
                    wolves.health_scale_tally += agents.genotypes[i][4];
                    wolves.speed_tally += agents.genotypes[i][5];
                    wolves.gestation_tally += agents.genotypes[i][6];
                    wolves.health_tally += agents.vitals[i][0]
                    wolves.hunger_tally += agents.vitals[i][1];
                    break;
                case 1:
                    sheep.count++;
                    sheep.body_size_tally += agents.genotypes[i][0];
                    sheep.sight_tally += agents.genotypes[i][1];
                    sheep.muscle_tally += agents.genotypes[i][2];
                    sheep.hunger_scale_tally += agents.genotypes[i][3];
                    sheep.health_scale_tally += agents.genotypes[i][4];
                    sheep.speed_tally += agents.genotypes[i][5];
                    sheep.gestation_tally += agents.genotypes[i][6];
                    sheep.health_tally += agents.vitals[i][0]
                    sheep.hunger_tally += agents.vitals[i][1];
                    break;
                case 2:
                    grass.count++;
                    grass.health_tally += agents.vitals[i][0]
                    break;
            }
        }

        this.logging_data.sheep.count.push(sheep.count);
        this.logging_data.wolves.count.push(wolves.count);
        this.logging_data.grass.count.push(grass.count);

        this.logging_data.sheep.body_size.push(sheep.body_size_tally / sheep.count);
        this.logging_data.sheep.sight_distance.push(sheep.sight_tally / sheep.count);
        this.logging_data.sheep.muscle_mass.push(sheep.muscle_tally / sheep.count);
        this.logging_data.sheep.hunger.push(sheep.hunger_tally / sheep.count);
        this.logging_data.sheep.hunger_rate.push(sheep.hunger_tally / sheep.count);
        this.logging_data.sheep.health.push(sheep.health_tally / sheep.count)
        this.logging_data.sheep.health_scale.push(sheep.health_scale_tally / sheep.count);
        this.logging_data.sheep.movement_speed.push(sheep.speed_tally / sheep.count);
        this.logging_data.sheep.gestation_duration.push(sheep.gestation_tally / sheep.count);

        this.logging_data.wolves.body_size.push(wolves.body_size_tally / wolves.count);
        this.logging_data.wolves.sight_distance.push(wolves.sight_tally / wolves.count);
        this.logging_data.wolves.muscle_mass.push(wolves.muscle_tally / wolves.count);
        this.logging_data.wolves.hunger.push(wolves.hunger_tally / wolves.count);
        this.logging_data.wolves.hunger_rate.push(wolves.hunger_tally / wolves.count);
        this.logging_data.wolves.health.push(wolves.health_tally / wolves.count)
        this.logging_data.wolves.health_scale.push(wolves.health_scale_tally / wolves.count);
        this.logging_data.wolves.movement_speed.push(wolves.speed_tally / wolves.count);
        this.logging_data.wolves.gestation_duration.push(wolves.gestation_tally / wolves.count);

        this.logging_data.grass.health.push(grass.health_tally / grass.count);

        this.logging_data.time.push(this.time);

        // TODO : properly log averages, then plot them on a canvas
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