/** Rows uploaded per texture call, which bounds the size of the staging buffer. */
const STAGING_ROWS = 32;

/**
 * A GPU texture that stores the most recent frequency rows as a ring.
 *
 * Every row is one analyser reading, the texture wraps vertically so the oldest row
 * always follows the newest one.
 */
export class FrequencyHistory {
  /** Number of rows kept before the oldest is overwritten. */
  readonly rowCount: number;

  private readonly gl: WebGL2RenderingContext;
  private readonly texture: WebGLTexture;
  private readonly binCount: number;
  /** Reusable upload buffer that holds copies of one row. */
  private readonly staging: Uint8Array<ArrayBuffer>;
  private newest: number;

  /**
   * Allocates the texture.
   *
   * `binCount` is the width of a row, `rowCount` is how many rows are kept.
   */
  constructor(gl: WebGL2RenderingContext, binCount: number, rowCount: number) {
    const texture = gl.createTexture();
    this.gl = gl;
    this.texture = texture;
    this.binCount = binCount;
    this.rowCount = rowCount;
    this.newest = rowCount - 1;
    this.staging = new Uint8Array(binCount * STAGING_ROWS);

    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texStorage2D(gl.TEXTURE_2D, 1, gl.R8, binCount, rowCount);
  }

  /** Index of the most recently written row. */
  get newestRow(): number {
    return this.newest;
  }

  /** Binds the texture to the active texture unit. */
  bind(): void {
    this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
  }

  /**
   * Writes `count` copies of `row` as the newest rows.
   *
   * Writes wrap around the end of the texture, so a large `count` is uploaded in several chunks.
   */
  push(row: Uint8Array, count: number): void {
    const { gl, binCount, rowCount, staging } = this;
    let remaining = Math.min(count, rowCount);

    this.bind();
    while (remaining > 0) {
      const start = (this.newest + 1) % rowCount;
      const chunk = Math.min(remaining, rowCount - start, STAGING_ROWS);
      for (let index = 0; index < chunk; index++) {
        staging.set(row, index * binCount);
      }
      gl.texSubImage2D(
        gl.TEXTURE_2D,
        0,
        0,
        start,
        binCount,
        chunk,
        gl.RED,
        gl.UNSIGNED_BYTE,
        staging,
      );
      this.newest = start + chunk - 1;
      remaining -= chunk;
    }
  }

  /** Deletes the GPU texture. */
  dispose(): void {
    this.gl.deleteTexture(this.texture);
  }
}
