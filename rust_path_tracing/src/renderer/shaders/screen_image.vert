#version 330 core

layout(location = 0) in vec2 pos;

out vec2 tex;

void main()
{
	tex = (pos + vec2(1)) * 0.5;
	gl_Position = vec4(pos, 0, 1);
}