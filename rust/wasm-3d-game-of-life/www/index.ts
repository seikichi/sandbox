import { memory } from "wasm-3d-game-of-life/wasm_3d_game_of_life_bg"
import { Universe, Cell } from "wasm-3d-game-of-life";
import * as THREE from 'three';

// wasm memory
const width = 20;
const height = 20;
const depth = 20;
const getIndex = (z: number, y: number, x: number): number => {
  return z * width * height + y * width + x;
};

// Load XR utils
(global as any).THREE = THREE;
require('three.xr.js');
const ANYTHREE = THREE as any;

// type definitions
interface Display {
  supportedRealities: {
    ar: boolean;
    vr: boolean;
  };
}

interface WebGLRendererWithXR extends THREE.WebGLRenderer {
  xr: any;
}

// variables
const universe = Universe.new();

let container, buttonsContainer: HTMLDivElement;
let scene: THREE.Scene;
let camera: THREE.PerspectiveCamera;
let renderer: WebGLRendererWithXR;
let activeRealityType: string = 'magicWindow';
const cells: THREE.Mesh[][][] = [];
const clock = new THREE.Clock();

(async () => {
  const displays: Display[] = await ANYTHREE.WebXRUtils.getDisplays();
  init(displays);
})();

const CELL_SIZE = 0.03; // 3cm
const CELL_LENS = 20;   // 20x20x20
const CELL_MARGIN = 1.2;
const CAMERA_DISTANCE = 1.0; // 1m

async function init(displays: Display[]) {
  container = document.createElement('div');
  document.body.appendChild(container);

  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera();
  scene.add(camera);
  camera.position.set(0, 0, 0);
  renderer = new THREE.WebGLRenderer({ alpha: true }) as WebGLRendererWithXR;
  renderer.autoClear = false;
  container.appendChild(renderer.domElement);

  {
    const size = CELL_SIZE * CELL_MARGIN;
    const offset = CELL_LENS * size / 2;
    for (let i = 0; i < CELL_LENS; i++) {
      cells.push([]);
      for (let j = 0; j < CELL_LENS; j++) {
        cells[i].push([]);
        for (let k = 0; k < CELL_LENS; k++) {
          const geometry = new THREE.BoxGeometry(CELL_SIZE, CELL_SIZE, CELL_SIZE);
          const material = new THREE.MeshNormalMaterial();

          const cell = new THREE.Mesh(geometry, material);
          cell.position.set(size * k - offset, size * j - offset, size * i - offset);
          (cell.material as any).opacity = 1.0;
          scene.add(cell);

          cells[i][j].push(cell);
        }
      }
    }

    camera.position.set(0, 0, (offset + CAMERA_DISTANCE));
  }

  // End custom content
  window.addEventListener('resize', onWindowResize, false);
  onWindowResize();

  const options = { AR_AUTOSTART: false } as any;
  renderer.xr = new ANYTHREE.WebXRManager(options, displays, renderer, camera, scene, update);
  renderer.xr.addEventListener('sessionStarted', sessionStarted);
  renderer.xr.addEventListener('sessionEnded', sessionEnded);

  console.log(renderer.xr.autoStarted);
  if (!renderer.xr.autoStarted) {
    buttonsContainer = document.createElement('div');
    buttonsContainer.id = 'buttonsContainer';
    buttonsContainer.style.position = 'absolute';
    buttonsContainer.style.bottom = '10%';
    buttonsContainer.style.width = '100%';
    buttonsContainer.style.textAlign = 'center';
    buttonsContainer.style.zIndex = '999';
    document.body.appendChild(buttonsContainer);
    addEnterButtons(displays);
  }

  renderer.animate(render);
}

function sessionStarted(data: any) {
  activeRealityType = data.session.realityType;
}

function sessionEnded(data: any) {
  activeRealityType = 'magicWindow';
}

function addEnterButtons(displays: Display[]) {
  for (var i = 0; i < displays.length; i++) {
    var display = displays[i];
    if (display.supportedRealities.vr) {
      buttonsContainer.appendChild(getEnterButton(display, 'vr'));
    }
    if (display.supportedRealities.ar) {
      buttonsContainer.appendChild(getEnterButton(display, 'ar'));
    }
  }
}

function getEnterButton(display: Display, reality: string) {
  // HMDs require the call to start presenting to occur due to a user input event, so make a button to trigger that
  var button = document.createElement('button');
  button.style.display = 'inline-block';
  button.style.margin = '5px';
  button.style.width = '120px';
  button.style.border = '0';
  button.style.padding = '8px';
  button.style.cursor = 'pointer';
  button.style.backgroundColor = '#000';
  button.style.color = '#fff';
  button.style.fontFamily = 'sans-serif';
  button.style.fontSize = '13px';
  button.style.fontStyle = 'normal';
  button.style.textAlign = 'center';
  if (reality === 'vr'){
    button.textContent = 'ENTER VR';
  } else {
    button.textContent = 'ENTER AR';
  }

  button.addEventListener('click', ev => {
    if (reality === 'ar') {
      if (!renderer.xr.sessionActive) {
        // Entering AR.
        button.textContent = 'EXIT AR';
        renderer.xr.startSession(display, reality, true);
      } else {
        // Exiting AR.
        button.textContent = 'ENTER AR';
        renderer.xr.endSession();
      }
    } else {
      buttonsContainer.style.display = 'none';
      renderer.xr.startPresenting();
    }
  });

  if (reality === 'vr') {
    window.addEventListener('vrdisplaypresentchange', (evt: any) => {
      // Polyfill places cameraActivateddisplay inside the detail property
      const display = evt.display || evt.detail.display;
      if (!display.isPresenting) {
        // Exiting VR.
        renderer.xr.endSession();
        buttonsContainer.style.display = 'block';
      }
    });
  }

  return button;
}

function onWindowResize() {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
}

// Called once per frame, before render, to give the app a chance to update this.scene
function update(_frame: any) {
  render();
}

function render() {
  switch (activeRealityType) {
    case 'magicWindow':
    case 'vr':
    case 'ar':
      var delta = clock.getDelta() * 60;
      // cube.rotation.y += delta * 0.01;
      break;
  }

  universe.tick();
  let actives = 0;
  const memory_cells = new Uint8Array(memory.buffer, universe.cells(), width * height * depth);
  for (let i = 0; i < CELL_LENS; i++) {
    for (let j = 0; j < CELL_LENS; j++) {
      for (let k = 0; k < CELL_LENS; k++) {
        const idx = getIndex(i, j, k);
        const dead = memory_cells[idx] === Cell.Dead;
        (cells[i][j][k].material as any).opacity = dead ? 0.0 : 1.0;
        actives += dead ? 0 : 1;
      }
    }
  }
  console.log(`active cells = ${actives}`);

  // if (!renderer.xr.sessionActive) {
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.render(scene, camera);
  // }
}
