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


THREE.MapControls = function (object, domElement) {

    let control = new OrbitControls(object, domElement);

    control.mouseButtons.LEFT = THREE.MOUSE.PAN;
    control.mouseButtons.RIGHT = THREE.MOUSE.ROTATE;

    control.touches.ONE = THREE.TOUCH.PAN;
    control.touches.TWO = THREE.TOUCH.DOLLY_ROTATE;

    return control
};

THREE.MapControls.prototype = Object.create(THREE.EventDispatcher.prototype);
THREE.MapControls.prototype.constructor = THREE.MapControls;


class Renderer {
    constructor(sheepNumber, wolfNumber, size) {
        log(`Renderer started with ${wolfNumber} wolves and ${sheepNumber} sheep.`)
        this.load_models();
        this.start = performance.now()
        this.size = size;
        this.sheepNumber = sheepNumber;
        this.wolfNumber = wolfNumber;

        this.renderer = new THREE.WebGLRenderer({ antialias: true });
        this.renderer.shadowMap.enabled = true;
        this.camera = new THREE.PerspectiveCamera(65, innerWidth / innerHeight, 0.1, 5000);
        this.camera.position.z = this.size / 3;
        this.camera.position.y = this.size / 3;
        this.camera.lookAt(new THREE.Vector3(0, 0, 0));

        this.fake_cam = this.camera.clone()
        this.scene = new THREE.Scene();
        /* this.camera_tracking_pivot = new THREE.Object3D();
        this.camera_tracking_pivot.name = "camera_tracking_pivot" 
        this.scene.add(this.camera_tracking_pivot);
        */

        this.load_lights();

        this.controller = THREE.MapControls(this.fake_cam, this.renderer.domElement);
        this.controller.screenSpacePanning = false;

        this.mousecaster = new THREE.Raycaster();
        this.three_mouse = new THREE.Vector2();
        this.mouse = { x: 0, y: 0 }

        this.ground = new THREE.Mesh(
            new THREE.PlaneGeometry(this.size, this.size),
            new THREE.MeshPhysicalMaterial({
                color: 0x002611,
                roughness: 1,
                specularIntensity: .2,
            })
        )
        this.ground.castShadow = true;
        this.ground.receiveShadow = true;
        this.ground.rotation.x = -Math.PI / 2
        this.scene.add(this.ground)

        this.setSize()
        document.body.appendChild(this.renderer.domElement)
        this.renderer.domElement.id = "three"

        this.load_models()

        /* this.render() */
    }

    update_agents(agents) {
        if (!this.done_loading) return;

        for(let i = 0; i < this.wolfNumber; i++) {
            let m = new THREE.Matrix4();
            m.setPosition(new THREE.Vector3(i * 10, 10, 0));
            this.wolves.setMatrixAt(i, m)
        }

        return

        /* let updated_instances = 0 */

        log(`Updating ${agents.positions.length} agents' matrices.`);
        for (let i = 0; i < agents.positions.length; i++) {
            const m = new THREE.Matrix4();
            const dead = agents.states[i] == 4;
            /* if(dead) log("dead") */
            switch (agents.types[i]) {
                case 0:
                    m.compose(
                        new THREE.Vector3(
                            agents.positions[i][0] - this.size / 2,
                            0,
                            agents.positions[i][1] - this.size / 2
                        ),
                        new THREE.Quaternion().setFromEuler(
                            new THREE.Euler(
                                0,
                                Math.atan2(agents.accelerations[i][0], agents.accelerations[i][1]),
                                dead ? 1.4 : 0
                            )
                        ),
                        new THREE.Vector3(agents.genotypes[i][0] / 10, agents.genotypes[i][0] / 10, agents.genotypes[i][0] / 10)
                    );
                    this.wolves.setMatrixAt(i,
                        m
                    )
                    let other = new THREE.Matrix4();
                    this.wolves.getMatrixAt(i, other)
                    log(other == m, other, m)
                    break;
                case 1:
                    m.compose(
                        new THREE.Vector3(
                            agents.positions[i][0] - this.size / 2,
                            0,
                            agents.positions[i][1] - this.size / 2
                        ),
                        new THREE.Quaternion().setFromEuler(
                            new THREE.Euler(
                                0,
                                Math.atan2(agents.accelerations[i][0], agents.accelerations[i][1]),
                                dead ? 1.4 : 0
                            )
                        ),
                        new THREE.Vector3(agents.genotypes[i][0] / 10, agents.genotypes[i][0] / 10, agents.genotypes[i][0] / 10)
                    );
                    this.sheep.setMatrixAt(i,
                        m
                    )
                    break;
                case 2:
                    m.compose(
                        new THREE.Vector3(
                            agents.positions[i][0] - this.size / 2,
                            0,
                            agents.positions[i][1] - this.size / 2
                        ),
                        new THREE.Quaternion().setFromEuler(
                            new THREE.Euler(
                                0,
                                agents.positions[i][0] * 3000,
                                0
                            )
                        ),
                        new THREE.Vector3(agents.vitals[i][0] / 100, agents.vitals[i][0] / 100, agents.vitals[i][0] / 100)
                    )
                    this.grass.setMatrixAt(i,
                        m
                    )
                    break;
            }
            log(`Updated instance ${i} with matrix ${m.elements}`)
        }
        /* log(`Updated ${updated_instances} instances.`) */
        this.sheep.instanceMatrix.needsUpdate = true;
        this.wolves.instanceMatrix.needsUpdate = true;
        this.grass.instanceMatrix.needsUpdate = true;
    }

    async load_models() {
        this.done_loading = false;
        const loader = new GLTFLoader();

        this.sheep_model = (await loader.loadAsync("./rsc/models/sheep.glb")).scene.children[0]
        /* log("sheep", this.sheep_model) */
        this.wolf_model = (await loader.loadAsync("./rsc/models/wolf.glb")).scene.children[0]
        /* log("wolf", this.wolf_model) */
        this.grass_model = (await loader.loadAsync("./rsc/models/grass.glb")).scene.children[0]
        /* log("grass", this.grass_model) */

        this.sheep = new THREE.InstancedMesh(
            this.sheep_model.geometry.scale(1.6, 1.6, 1.6),
            this.sheep_model.material,
            this.sheepNumber
        )
        this.sheep.castShadow = true;
        this.sheep.receiveShadow = true;
        this.scene.add(this.sheep)

        this.wolves = new THREE.InstancedMesh(
            this.wolf_model.geometry.scale(2, 2, 2),
            this.wolf_model.material,
            this.wolfNumber
        )
        this.wolves.castShadow = true;
        this.wolves.receiveShadow = true;
        this.scene.add(this.wolves)

        this.grass = new THREE.InstancedMesh(
            this.grass_model.geometry.scale(4, 4, 4),
            this.grass_model.material,
            MAX_GRASS
        )
        this.grass.castShadow = true;
        this.grass.receiveShadow = true;
        this.scene.add(this.grass)

        // Put each matrix's children at nil position
        const nil = new THREE.Matrix4().makeTranslation(0, 10, 0);
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
        this.ground.material.envMapIntensity = 0.5;
        /* log(this.ground.material) */

        const texLoader = new THREE.TextureLoader();

        const circle_tex = await (texLoader.loadAsync("./rsc/textures/circle.png"));
        this.selection_circle = new THREE.Mesh(
            new THREE.PlaneGeometry(1, 1),
            new THREE.MeshBasicMaterial({
                map: circle_tex,
                transparent: true,
                opacity: 0.5,
                side: THREE.DoubleSide
            }));
        this.selection_circle.rotation.x = Math.PI / 2
        this.selection_circle.scale.multiplyScalar(15)
        /* this.selection_circle.visible = false; */
        this.selection_circle.position.y = 1;
        this.scene.add(this.selection_circle);

        const grass_tex = await (texLoader.loadAsync("./rsc/textures/grass.jpg"));
        grass_tex.repeat.x = 4;
        grass_tex.repeat.y = 4;
        grass_tex.wrapS = THREE.RepeatWrapping;
        grass_tex.wrapT = THREE.RepeatWrapping;
        this.ground.material.map = grass_tex;
        this.ground.material.needsUpdate = true;

        this.done_loading = true;
    }

    async load_lights() {
        this.sun = new THREE.DirectionalLight(0xffffff, 1.2);
        this.sun.castShadow = true;
        this.scene.add(this.sun);
        this.sun.position.x = this.size;
        this.sun.position.y = this.size;
        this.sun.lookAt(new THREE.Vector3(0, 0, 0));
        this.sun.shadow.camera.far = 5000;
        this.sun.shadow.camera.bottom = -this.size / 2;
        this.sun.shadow.camera.top = this.size / 2;
        this.sun.shadow.camera.left = -this.size / 2;
        this.sun.shadow.camera.right = this.size / 2;
        this.sun.shadow.bias = 0.01

        /* this.scene.add(new THREE.CameraHelper(this.sun.shadow.camera)); */

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
        if (this.done_loading) {
            this.update_agents(agents)
        }
        this.renderer.render(this.scene, this.camera);
    }
}

export default Renderer