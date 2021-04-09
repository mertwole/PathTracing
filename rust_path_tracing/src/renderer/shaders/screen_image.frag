#version 330 core

out vec4 color;

uniform sampler2D screen_image;

in vec2 tex;

void main()
{
	color = vec4(texture(screen_image, tex).rgb, 1.0);
}