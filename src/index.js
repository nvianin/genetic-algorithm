const THREE = require("three")

class Renderer {
    constructor(sheepNumber, wolfNumber) {
        log(`Renderer started with ${wolfNumber} wolves and ${sheepNumber} sheep.`)
        this.start = performance.now()

        this.renderer = new THREE.WebGLRenderer();
        this.camera = new THREE.PerspectiveCamera();
        this.scene = new THREE.Scene();

        this.setSize()
        document.body.appendChild(this.renderer.domElement)

        this.sheep = new THREE.InstancedMesh(
            new THREE.SphereGeometry(.5, 16, 32),
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
    }

    setSize(width = innerWidth, height = innerHeight) {
        this.renderer.setSize(width, height);
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }

    render() {
        this.renderer.render(this.scene, this.camera);
    }
}

export default Renderer