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

        this.sheeps = new THREE.InstancedMesh(
            new THREE.SphereGeometry(.5, 16, 32),
            new THREE.MeshBasicMaterial({
                color: 0x00ff00
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
    }

    setSize(width = innerWidth, height = innerHeight) {
        this.renderer.setSize(width, height);
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }

    render() {
        requestAnimationFrame(this.render.bind(this));
        this.renderer.render(this.scene, this.camera);
    }
}

export default Renderer