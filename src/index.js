const THREE = require("three")
const { GLTFLoader } = require("three/examples/jsm/loaders/GLTFLoader")
const { OrbitControls } = require("three/examples/jsm/controls/OrbitControls")

class Renderer {
    constructor(sheepNumber, wolfNumber, size) {
        log(`Renderer started with ${wolfNumber} wolves and ${sheepNumber} sheep.`)
        this.load_models();
        this.start = performance.now()
        this.size = size;
        this.sheepNumber = sheepNumber;
        this.wolfNumber = wolfNumber;

        this.renderer = new THREE.WebGLRenderer();
        this.camera = new THREE.PerspectiveCamera();
        this.camera.y = 100;
        this.camera.lookAt(new THREE.Vector3(0, 0, 0));
        this.scene = new THREE.Scene();

        this.controller = new OrbitControls(this.camera, this.renderer.domElement);

        this.ground = new THREE.Mesh(
            new THREE.PlaneGeometry(this.size, this.size),
            new THREE.MeshBasicMaterial({
                color: 0x00ff00
            })
        )
        this.ground.rotation.x = -Math.PI / 2
        this.scene.add(this.ground)

        this.setSize()
        document.body.appendChild(this.renderer.domElement)
        this.renderer.domElement.id = "three"

        this.load_models()

        /* this.render() */
    }

    async load_models() {
        const loader = new GLTFLoader();

        this.sheep_model = (await loader.loadAsync("./rsc/models/sheep.glb")).scene.children[0]
        log("sheep", this.sheep_model)
        this.wolf_model = (await loader.loadAsync("./rsc/models/wolf.glb")).scene.children[0]
        log("wolf", this.wolf_model)
        this.grass_model = (await loader.loadAsync("./rsc/models/grass.glb")).scene.children[0]
        log("grass", this.grass_model)

        this.sheep = new THREE.InstancedMesh(
            this.sheep_model.geometry,
            this.sheep_model.material,
            this.sheepNumber
        )

        this.wolves = new THREE.InstancedMesh(
            new THREE.SphereGeometry(.25, 16, 32),
            new THREE.MeshBasicMaterial({
                color: 0xff0000
            }),
            this.wolfNumber
        )

        this.grass = new THREE.InstancedMesh(
            new THREE.BoxGeometry(.1, 1, .1),
            new THREE.MeshBasicMaterial({
                color: 0x00ff00
            }),
            1024
        )

        this.render();
    }

    setSize(width = innerWidth, height = innerHeight) {
        this.renderer.setSize(width, height);
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }


    render() {
        /* log("rendering") */
        this.controller.update()
        this.renderer.render(this.scene, this.camera);
        requestAnimationFrame(this.render.bind(this));
    }
}

export default Renderer