#version 430 core



layout(location = 0) uniform mat4 uniform_Projection;
layout(location = 1) uniform mat4 uniform_View;

layout(location = 0) in vec3 in_Position;
layout(location = 1) in vec2 in_TexCoord;
layout(location = 2) in vec3 in_Normal;

layout(location = 0) out vec3 out_Position;
layout(location = 1) out vec2 out_TexCoord;
layout(location = 2) out vec3 out_Normal;

void main() {
	vec4 viewPosition = uniform_View * vec4(in_Position, 1.0);
	gl_Position = uniform_Projection * viewPosition;
	out_Position = viewPosition.xyz / viewPosition.w;
	out_TexCoord = in_TexCoord;
	out_Normal = mat3(uniform_View) * in_Normal;
}
