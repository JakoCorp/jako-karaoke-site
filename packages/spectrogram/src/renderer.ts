import { mat4 } from "gl-matrix";

import { FrequencyHistory } from "./frequency-history";
import fragmentSource from "./shaders/fragment.glsl?raw";
import vertexSource from "./shaders/vertex.glsl?raw";

/** Width and height of the mesh in world units, before the aspect ratio is applied. */
const MESH_SIZE = 10;
/** Height of the mesh at full power in world units. */
const HEIGHT_SCALE = MESH_SIZE / 6;
/** Rows kept in the history texture, so the visible time span is this divided by the rows per second. */
const HISTORY_ROWS = 2048;
/** Base of the exponential curve that maps mesh height to frequency bins. */
const FREQUENCY_AXIS_BASE = 225;
/** Texel shift that hides the seam between the oldest and newest rows. */
const SEAM_TEXEL_SHIFT = 2;
/** Vertical field of view of the camera. */
const FOV_Y_RADIANS = (52 * Math.PI) / 180;
/** Distance from the camera at which the whole mesh height fits the field of view. */
const CAMERA_DISTANCE = 10.52;
/** Two floats for the position followed by two for the texture coordinate. */
const FLOATS_PER_VERTEX = 4;

/** Number of mesh vertices along each axis. */
export interface MeshDensity {
  /** Columns along the time axis, each drawn as one instance. */
  columns: number;
  /** Vertices along the frequency axis. */
  rows: number;
}

/**
 * Draws the scrolling spectrogram mesh with WebGL2.
 *
 * The mesh is a single column of triangle strip vertices drawn once per time step.
 * The vertex shader reads the frequency history texture to displace and color it.
 * The renderer owns the WebGL context of its canvas and loses it on {@link SpectrogramRenderer.dispose}.
 */
export class SpectrogramRenderer {
  private readonly history: FrequencyHistory;
  private readonly canvas: HTMLCanvasElement;
  private readonly gl: WebGL2RenderingContext;
  private readonly program: WebGLProgram;
  private readonly vertexArray: WebGLVertexArrayObject;
  private readonly vertexBuffer: WebGLBuffer;
  private readonly columns: number;
  private readonly vertexCount: number;
  private readonly uniforms: {
    mvpMatrix: WebGLUniformLocation;
    historyOffset: WebGLUniformLocation;
  };
  private readonly mvpMatrix = mat4.create();

  /**
   * Builds the GPU resources for a canvas.
   *
   * `binCount` is the number of frequency bins in each analyser reading.
   * Throws when WebGL2 or vertex texture reads are unavailable, or when the shaders fail to build.
   */
  constructor(canvas: HTMLCanvasElement, density: MeshDensity, binCount: number) {
    const gl = canvas.getContext("webgl2", { antialias: false, alpha: false });
    if (gl === null) throw new Error("WebGL2 is not available.");
    const vertexTextureUnits: unknown = gl.getParameter(gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS);
    if (typeof vertexTextureUnits !== "number" || vertexTextureUnits < 1) {
      throw new Error("Vertex shaders cannot read textures on this device.");
    }

    this.canvas = canvas;
    this.gl = gl;
    this.columns = density.columns;
    this.vertexCount = density.rows * 2;
    this.history = new FrequencyHistory(gl, binCount, HISTORY_ROWS);

    this.program = createProgram(gl);
    gl.useProgram(this.program);

    this.vertexArray = gl.createVertexArray();
    this.vertexBuffer = gl.createBuffer();
    gl.bindVertexArray(this.vertexArray);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, createMesh(density), gl.STATIC_DRAW);

    const stride = FLOATS_PER_VERTEX * Float32Array.BYTES_PER_ELEMENT;
    const textureOffset = 2 * Float32Array.BYTES_PER_ELEMENT;
    const position = gl.getAttribLocation(this.program, "aPosition");
    gl.enableVertexAttribArray(position);
    gl.vertexAttribPointer(position, 2, gl.FLOAT, false, stride, 0);
    const texCoord = gl.getAttribLocation(this.program, "aTexCoord");
    gl.enableVertexAttribArray(texCoord);
    gl.vertexAttribPointer(texCoord, 2, gl.FLOAT, false, stride, textureOffset);

    gl.uniform1i(requireUniform(gl, this.program, "history"), 0);
    gl.uniform1f(requireUniform(gl, this.program, "columnWidth"), MESH_SIZE / density.columns);
    gl.uniform1f(
      requireUniform(gl, this.program, "columnTexelStep"),
      columnTexelStep(density.columns),
    );
    gl.uniform1f(requireUniform(gl, this.program, "heightScale"), HEIGHT_SCALE);
    this.uniforms = {
      mvpMatrix: requireUniform(gl, this.program, "mvpMatrix"),
      historyOffset: requireUniform(gl, this.program, "historyOffset"),
    };

    gl.clearColor(0, 0, 0, 1);
    gl.enable(gl.DEPTH_TEST);
    this.history.bind();
  }

  /** Resizes the drawing buffer, `width` and `height` are in device pixels. */
  resize(width: number, height: number): void {
    if (width === 0 || height === 0) return;

    this.canvas.width = width;
    this.canvas.height = height;
    this.gl.viewport(0, 0, width, height);

    const aspect = width / height;
    mat4.perspective(this.mvpMatrix, FOV_Y_RADIANS, aspect, 1, 100);
    mat4.translate(this.mvpMatrix, this.mvpMatrix, [0, 0, -CAMERA_DISTANCE]);
    mat4.scale(this.mvpMatrix, this.mvpMatrix, [aspect, 1, 1]);
    this.gl.uniformMatrix4fv(this.uniforms.mvpMatrix, false, this.mvpMatrix);
  }

  /** Adds `count` copies of one analyser reading as the newest rows of the history. */
  pushRows(row: Uint8Array, count: number): void {
    this.history.push(row, count);
  }

  /** Draws the current contents of the history texture. */
  render(): void {
    const { gl, columns, history } = this;

    const columnSteps = columns - 1;
    const offset =
      Math.floor((history.newestRow / (history.rowCount - 1)) * columnSteps) / columnSteps;
    gl.uniform1f(this.uniforms.historyOffset, offset);

    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, this.vertexCount, columnSteps);
  }

  /** Releases every GPU resource and the WebGL context. */
  dispose(): void {
    const { gl } = this;
    this.history.dispose();
    gl.deleteBuffer(this.vertexBuffer);
    gl.deleteVertexArray(this.vertexArray);
    gl.deleteProgram(this.program);
    gl.getExtension("WEBGL_lose_context")?.loseContext();
  }
}

/** Returns the texture distance between neighbouring columns of a mesh. */
function columnTexelStep(columns: number): number {
  return 1 / (columns + SEAM_TEXEL_SHIFT - 1);
}

/**
 * Builds the interleaved vertices of one mesh column as a triangle strip.
 *
 * Each vertex holds its position followed by the texture coordinate that selects its frequency bin and time step.
 */
function createMesh({ columns, rows }: MeshDensity): Float32Array {
  const vertices = new Float32Array(rows * 2 * FLOATS_PER_VERTEX);
  const texelStep = columnTexelStep(columns);
  let cursor = 0;
  for (let row = 0; row < rows; row++) {
    const frequency = FREQUENCY_AXIS_BASE ** (row / (rows - 1) - 1);
    for (let column = 0; column < 2; column++) {
      vertices[cursor++] = (MESH_SIZE * (column - columns / 2)) / columns;
      vertices[cursor++] = (MESH_SIZE * (row - rows / 2)) / rows;
      vertices[cursor++] = frequency;
      vertices[cursor++] = (column + SEAM_TEXEL_SHIFT) * texelStep;
    }
  }
  return vertices;
}

/** Compiles and links the spectrogram shaders, throws with the driver log on failure. */
function createProgram(gl: WebGL2RenderingContext): WebGLProgram {
  const program = gl.createProgram();
  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  gl.deleteShader(vertexShader);
  gl.deleteShader(fragmentShader);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(program);
    gl.deleteProgram(program);
    throw new Error(`Spectrogram program failed to link: ${log}`);
  }
  return program;
}

function compileShader(gl: WebGL2RenderingContext, type: number, source: string): WebGLShader {
  const shader = gl.createShader(type);
  if (shader === null) throw new Error("Spectrogram shader could not be created.");
  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error(`Spectrogram shader failed to compile: ${log}`);
  }
  return shader;
}

/** Finds a uniform the program must declare, a missing one means the shader and renderer disagree. */
function requireUniform(
  gl: WebGL2RenderingContext,
  program: WebGLProgram,
  name: string,
): WebGLUniformLocation {
  const location = gl.getUniformLocation(program, name);
  if (location === null) throw new Error(`Spectrogram uniform ${name} is missing.`);
  return location;
}
