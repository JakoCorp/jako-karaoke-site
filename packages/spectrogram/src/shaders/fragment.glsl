#version 300 es
precision highp float;

// Samples the history again per pixel so detail is not limited by the mesh density.

in vec3 vColor;
in vec2 vTexCoord;

uniform sampler2D history;

out vec4 fragColor;

void main() {
  float power = texture(history, vTexCoord).r;
  fragColor = vec4(power * vColor, 1.0);
}
