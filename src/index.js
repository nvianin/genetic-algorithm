const THREE = require("three")
const { GLTFLoader } = require("three/examples/jsm/loaders/GLTFLoader")
/* const {MapControls} = require("three/examples/jsm/controls/") */

class Renderer {
    constructor(sheepNumber, wolfNumber, size) {
        log(`Renderer started with ${wolfNumber} wolves and ${sheepNumber} sheep.`)
        this.load_models();
        this.start = performance.now()
        this.size = size;

        this.renderer = new THREE.WebGLRenderer();
        this.camera = new THREE.PerspectiveCamera();
        this.scene = new THREE.Scene();

        /* this.controller = new  */

        this.setSize()
        document.body.appendChild(this.renderer.domElement)
        this.renderer.domElement.id = "three"

        this.sheep = new THREE.InstancedMesh(
            new THREE.SphereGeometry(),
            new THREE.MeshBasicMaterial({
                color: 0xaaaaaa
            }),
            sheepNumber
        )

        this.wolves = new THREE.InstancedMesh(
            new THREE.SphereGeometry(.25, 16, 32),
            new THREE.MeshBasicMaterial({
                color: 0xff0000
            }),
            wolfNumber
        )

        this.grass = new THREE.InstancedMesh(
            new THREE.BoxGeometry(.1, 1, .1),
            new THREE.MeshBasicMaterial({
                color: 0x00ff00
            }),
            1024
        )

        this.render()
    }

    async load_models() {
        const loader = new GLTFLoader();

        this.sheep_model = (await loader.loadAsync("./rsc/models/sheep.glb")).scene.children[0]
        log("sheep", this.sheep_model)
        this.wolf_model = (await loader.loadAsync("./rsc/models/wolf.glb")).scene.children[0]
        log("wolf", this.wolf_model)
        this.grass_model = (await loader.loadAsync("./rsc/models/grass.glb")).scene.children[0]
        log("grass", this.grass_model)
    }

    setSize(width = innerWidth, height = innerHeight) {
        this.renderer.setSize(width, height);
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }


    render() {
        /* log("rendering") */
        this.renderer.render(this.scene, this.camera);
        requestAnimationFrame(this.render.bind(this));
    }
}

export default Renderer