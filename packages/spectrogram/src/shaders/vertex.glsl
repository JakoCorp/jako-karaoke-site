#version 300 es

// Each instance is one column of the mesh, displaced by the power stored in the history texture.

// Position of the vertex within the first column of the mesh.
in vec2 aPosition;
// Frequency bin on x and time step on y, both normalized to the texture.
in vec2 aTexCoord;

// Power of every frequency bin over time, one row per time step.
uniform sampler2D history;
uniform mat4 mvpMatrix;
// Shifts the lookup so the newest row lands on the right edge of the mesh.
uniform float historyOffset;
// Distance between neighbouring columns in the mesh.
uniform float columnWidth;
// Distance between neighbouring columns in the texture.
uniform float columnTexelStep;
// Height of the mesh at full power.
uniform float heightScale;

out vec3 vColor;
out vec2 vTexCoord;

// Based on an answer by Sam Hocevar on Stack Overflow (https://stackoverflow.com/a/17897228), licensed under the WTFPL.
vec3 hsv2rgb(vec3 hsv) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(hsv.xxx + K.xyz) * 6.0 - K.www);
  return hsv.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), hsv.y);
}

void main() {
  float column = float(gl_InstanceID);
  vTexCoord = vec2(aTexCoord.x, aTexCoord.y + historyOffset + column * columnTexelStep);

  float power = texture(history, vTexCoord).r;
  vec4 position = vec4(aPosition.x + column * columnWidth, aPosition.y, heightScale * power, 1.0);
  gl_Position = mvpMatrix * position;

  vColor = hsv2rgb(vec3(1.0 - power, 1.0, 1.0));
}
