const THREE = require("three")
const {
    GLTFLoader
} = require("three/examples/jsm/loaders/GLTFLoader")
const {
    OrbitControls
} = require("three/examples/jsm/controls/OrbitControls")
const {
    EXRLoader
} = require("three/examples/jsm/loaders/EXRLoader")

const MAX_GRASS = 1024;

class Renderer {
    constructor(sheepNumber, wolfNumber, size) {
        log(`Renderer started with ${wolfNumber} wolves and ${sheepNumber} sheep.`)
        this.load_models();
        this.start = performance.now()
        this.size = size;
        this.sheepNumber = sheepNumber;
        this.wolfNumber = wolfNumber;

        this.renderer = new THREE.WebGLRenderer();
        this.camera = new THREE.PerspectiveCamera(65, innerWidth / innerHeight, 0.1, 5000);
        this.camera.position.z = this.size;
        this.camera.position.y = this.size;
        this.camera.lookAt(new THREE.Vector3(0, 0, 0));
        this.scene = new THREE.Scene();

        this.load_lights();

        this.controller = new OrbitControls(this.camera, this.renderer.domElement);

        this.ground = new THREE.Mesh(
            new THREE.PlaneGeometry(this.size, this.size),
            new THREE.MeshPhysicalMaterial({
                color: 0x00ff00,
            })
        )
        this.ground.rotation.x = -Math.PI / 2
        this.scene.add(this.ground)

        this.setSize()
        document.body.appendChild(this.renderer.domElement)
        this.renderer.domElement.id = "three"

        this.load_models()

        /* setInterval(() => {
            window.location.reload()
        }, 10000); */

        /* this.render() */
    }

    update_agents(agents) {
        if (!this.done_loading) return;

        /* log(`Updating ${agents.positions.length} agents.`); */
        for (let i = 0; i < agents.positions.length; i++) {
            switch (agents.types[i]) {
                case 0:
                    this.wolves.setMatrixAt(i, new THREE.Matrix4()
                        .makeTranslation(agents.positions[i][0] - this.size / 2,
                            0,
                            agents.positions[i][1] - this.size / 2))
                    break;
                case 1:
                    this.sheep.setMatrixAt(i, new THREE.Matrix4()
                        .makeTranslation(agents.positions[i][0] - this.size / 2,
                            0,
                            agents.positions[i][1] - this.size / 2))
                    break;
                case 2:
                    this.grass.setMatrixAt(i, new THREE.Matrix4()
                        .makeTranslation(agents.positions[i][0] - this.size / 2,
                            0,
                            agents.positions[i][1] - this.size / 2))
                    break;
            }
        }
        this.sheep.instanceMatrix.needsUpdate = true;
        this.wolves.instanceMatrix.needsUpdate = true;
        this.grass.instanceMatrix.needsUpdate = true;
    }

    async load_models() {
        this.done_loading = false;
        const loader = new GLTFLoader();

        this.sheep_model = (await loader.loadAsync("./rsc/models/sheep.glb")).scene.children[0]
        log("sheep", this.sheep_model)
        this.wolf_model = (await loader.loadAsync("./rsc/models/wolf.glb")).scene.children[0]
        log("wolf", this.wolf_model)
        this.grass_model = (await loader.loadAsync("./rsc/models/grass.glb")).scene.children[0]
        log("grass", this.grass_model)

        this.sheep = new THREE.InstancedMesh(
            this.sheep_model.geometry.scale(4, 4, 4),
            this.sheep_model.material,
            this.sheepNumber
        )
        this.scene.add(this.sheep)

        this.wolves = new THREE.InstancedMesh(
            this.wolf_model.geometry.scale(4, 4, 4),
            this.wolf_model.material,
            this.wolfNumber
        )
        this.scene.add(this.wolves)

        this.grass = new THREE.InstancedMesh(
            this.grass_model.geometry.scale(16, 16, 16),
            this.grass_model.material,
            MAX_GRASS
        )
        this.scene.add(this.grass)

        // Put each matrix's children at nil position
        const nil = new THREE.Matrix4().makeTranslation(0, -10000, 0);
        for (let i = 0; i < this.sheepNumber; i++) {
            this.sheep.setMatrixAt(i, nil);
        }
        for (let i = 0; i < this.wolfNumber; i++) {
            this.wolves.setMatrixAt(i, nil);
        }
        for (let i = 0; i < this.grass.count; i++) {
            this.grass.setMatrixAt(i, nil)
        }

        this.wolf_model.material.envMap = this.exr;
        this.sheep_model.material.envMap = this.exr;
        this.grass_model.material.envMap = this.exr;
        this.ground.material.envMap = this.exr;
        log(this.ground.material)

        this.done_loading = true;

        this.render();
    }

    async load_lights() {
        this.sun = new THREE.DirectionalLight(0xffffff, 4.5);
        this.scene.add(this.sun);
        this.sun.position.x = this.size;
        this.sun.position.y = this.size;
        this.sun.lookAt(new THREE.Vector3(0, 0, 0));

        const exr_loader = new EXRLoader();
        this.exr = (await exr_loader.loadAsync("./rsc/textures/scythian_tombs_2_1k.exr"));
        this.exr.mapping = THREE.EquirectangularReflectionMapping;
        this.scene.background = this.exr;
    }

    setSize(width = innerWidth, height = innerHeight) {
        this.renderer.setSize(width, height);
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }


    render(agents) {
        /* log("rendering") */
        /* this.controller.update() */
        this.update_agents(agents)
        this.renderer.render(this.scene, this.camera);
    }
}

export default Renderer