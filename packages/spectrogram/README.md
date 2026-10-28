# @jako/spectrogram

A scrolling 3D spectrogram of a Web Audio node, drawn on a canvas with WebGL2.

This package is based on [kcpr-swrm/spectrogram](https://github.com/kcpr-swrm/spectrogram) and is used with the author's permission. It reimplements that project's WebGL2 rendering approach as a framework agnostic TypeScript library, without its integration code for the original karaoke site.

It is a private workspace package. The source is consumed directly through `exports`, so there is no build step.

## Requirements

- A bundler that supports Vite's `?raw` import suffix, because the shaders are `.glsl` files imported as strings.
- A browser with WebGL2 where vertex shaders can read textures. The constructor throws otherwise.
- An `AudioNode` to analyse. The package creates its own `AnalyserNode` and only reads from the node, so your own audio routing is untouched.
- For a `MediaElementAudioSourceNode`, the media needs `crossOrigin="anonymous"` and CORS headers. Cross origin media without them outputs silence.
- A running `AudioContext`, which browsers usually only allow after a user gesture.

## Usage

```ts
import { Spectrogram } from "@jako/spectrogram";

const spectrogram = new Spectrogram(canvas, { source: mediaElementSource });
spectrogram.resize(canvas.clientWidth, canvas.clientHeight, devicePixelRatio);
spectrogram.start();

// Later, when the audio pauses.
spectrogram.stop();

// When the spectrogram is no longer needed.
spectrogram.dispose();
```

- `start()` and `stop()` control the animation loop. New audio scrolls in from the right, and `stop()` keeps the current image.
- `resize(width, height, devicePixelRatio)` takes CSS pixels. The drawing buffer does not follow the canvas size, so call it whenever the canvas changes size, for example from a `ResizeObserver`.
- `dispose()` disconnects from the audio node and frees the GPU resources. The canvas loses its WebGL context, so it cannot be reused for another instance. Create a fresh canvas instead.

## Options

| Option          | Default   | Description                                                                                           |
| --------------- | --------- | ----------------------------------------------------------------------------------------------------- |
| `source`        | required  | The audio node to analyse.                                                                            |
| `meshDensity`   | 896 x 768 | Mesh `columns` and `rows`. Lower values render faster on weak GPUs.                                   |
| `fftSize`       | 4096      | FFT window size, a power of two from 32 to 32768. It sets the frequency resolution.                   |
| `rowsPerSecond` | 240       | Time resolution of the history. It sets the scroll speed, about 8.5 seconds are shown at the default. |

The scroll speed depends on `rowsPerSecond` only, not on the display refresh rate.

## Credits

- Rendering approach by [kcpr-swrm](https://github.com/kcpr-swrm/spectrogram).
- The HSV to RGB shader function is based on an [answer by Sam Hocevar](https://stackoverflow.com/a/17897228), licensed under the WTFPL.
