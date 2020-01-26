#version 440 core
out vec4 color;

in vec2 tex_coord;

uniform sampler2D tex;

void main()
{
	color = texture(tex, tex_coord);
	color.x = pow(color.x, 1 / 2.2);
	color.y = pow(color.y, 1 / 2.2);
	color.z = pow(color.z, 1 / 2.2);
}

	