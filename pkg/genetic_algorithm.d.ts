/* tslint:disable */
/* eslint-disable */
/**
*/
export function greet(): void;
/**
*/
export class SerializedVector2 {
  free(): void;
}
/**
*/
export class Test {
  free(): void;
}
/**
*/
export class World {
  free(): void;
/**
* @param {number} sheep_num
* @param {number} wolf_num
* @param {number} size
*/
  constructor(sheep_num: number, wolf_num: number, size: number);
/**
* @param {boolean} optimized
* @param {number} time
*/
  step(optimized: boolean, time: number): void;
/**
* @returns {number}
*/
  test(): number;
/**
* @returns {any}
*/
  get_quadtree(): any;
/**
* @returns {any}
*/
  get_agents(): any;
/**
* @param {number} mouse_x
* @param {number} mouse_y
* @returns {any}
*/
  activate(mouse_x: number, mouse_y: number): any;
/**
* @param {number} x
* @param {number} y
* @param {number} radius
* @returns {any}
*/
  get_agents_in_radius(x: number, y: number, radius: number): any;
/**
* @param {number} x
* @param {number} y
* @returns {number}
*/
  get_noise(x: number, y: number): number;
/**
* @returns {number}
*/
  static noise_scale(): number;
/**
*/
  seed: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_serializedvector2_free: (a: number) => void;
  readonly __wbg_world_free: (a: number) => void;
  readonly __wbg_get_world_seed: (a: number) => number;
  readonly __wbg_set_world_seed: (a: number, b: number) => void;
  readonly world_new: (a: number, b: number, c: number) => number;
  readonly world_step: (a: number, b: number, c: number) => void;
  readonly world_test: (a: number) => number;
  readonly world_get_quadtree: (a: number) => number;
  readonly world_get_agents: (a: number) => number;
  readonly world_activate: (a: number, b: number, c: number) => number;
  readonly world_get_agents_in_radius: (a: number, b: number, c: number, d: number) => number;
  readonly world_get_noise: (a: number, b: number, c: number) => number;
  readonly world_noise_scale: () => number;
  readonly __wbg_test_free: (a: number) => void;
  readonly greet: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
