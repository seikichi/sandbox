import { memory } from "wasm-3d-game-of-life/wasm_3d_game_of_life_bg";
import { Universe, Cell } from "wasm-3d-game-of-life";

import * as THREE from "three";
global.THREE = THREE; // for webxr-polyfill

import "./webxr-polyfill";
import { XRExampleBase } from "./common";

// constants
const CELL_SIZE = 0.0025; // 0.25cm
const CELL_LENS = 64; // 64x64
const CELL_MARGIN = 1.2;

// main
setTimeout(() => {
  try {
    window.pageApp = new ARSimplestExample(document.getElementById("target"));
  } catch (e) {
    console.error("page error", e);
  }
}, 1000);

class ARSimplestExample extends XRExampleBase {
  constructor(domElement) {
    super(domElement, false);
    this._tapEventData = null;
    this.el.addEventListener(
      "touchstart",
      this._onTouchStart.bind(this),
      false
    );

    this.universe = Universe.new(CELL_LENS, CELL_LENS);
    this.board = new THREE.Group();
    this.cells = [];

    const size = CELL_SIZE * CELL_MARGIN;
    const offset = (CELL_LENS * size) / 2;

    for (let i = 0; i < CELL_LENS; i++) {
      this.cells.push([]);
      for (let j = 0; j < CELL_LENS; j++) {
        const geometry = new THREE.BoxGeometry(CELL_SIZE, CELL_SIZE, CELL_SIZE);
        const material = new THREE.MeshNormalMaterial();
        const cell = new THREE.Mesh(geometry, material);
        cell.position.set(size * i - offset, 0, size * j - offset);

        this.cells[i].push(cell);
        this.board.add(cell);
      }
    }

    this.board.geometry.rotate(Math.PI / 2);
  }

  async updateScene(frame) {
    // update universe
    this.universe.tick();
    const width = this.universe.width();
    const height = this.universe.width();
    const memory_cells = new Uint8Array(
      memory.buffer,
      this.universe.cells(),
      width * height
    );

    for (let i = 0; i < height; i++) {
      for (let j = 0; j < width; j++) {
        const idx = i * width + j;
        const dead = memory_cells[idx] === Cell.Dead;
        this.cells[i][j].material.opacity = dead ? 0.0 : 1.0;
      }
    }

    // handle tap
    if (this._tapEventData === null) {
      return;
    }
    const x = this._tapEventData[0];
    const y = this._tapEventData[1];
    this._tapEventData = null;
    // Attempt a hit test using the normalized screen coordinates
    const anchorOffset = await frame.findAnchor(x, y);
    if (anchorOffset === null) {
      return;
    }
    this.removeAnchoredNode(this.board);
    this.addAnchoredNode(anchorOffset, this.board);
  }

  _onTouchStart(ev) {
    if (!ev.touches || ev.touches.length === 0) {
      console.error("No touches on touch event", ev);
      return;
    }
    this._tapEventData = [
      ev.touches[0].clientX / global.innerWidth,
      ev.touches[0].clientY / global.innerHeight
    ];
  }
}
