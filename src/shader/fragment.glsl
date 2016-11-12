#version 120

varying vec2 v_tex_coords;

uniform sampler2D tex;

void main() {
  float c = texture2D(tex, v_tex_coords).r;
  gl_FragColor = vec4(c, c, c, 1.0);
}
