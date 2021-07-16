#version 430 core



layout(location = 1) uniform mat4 uniform_View;
layout(binding = 0) uniform usampler2D texture_FaceMap;
layout(binding = 1) uniform sampler2D texture_Albedo;

layout(std430, binding = 0) readonly buffer positionBlock {
	float storage_PositionComponents[];
};

layout(std430, binding = 1) readonly buffer normalBlock {
	float storage_NormalComponents[];
};

layout(std430, binding = 2) readonly buffer faceBlock {
	uint storage_Indices[];
};

layout(std430) struct solver {
	mat2 matrix;
	vec2 texCoord;
};

layout(std430, binding = 3) readonly buffer solverBlock {
	solver storage_Solvers[];
};

layout(location = 0) in vec3 in_Position;
layout(location = 1) in vec2 in_TexCoord;
layout(location = 2) in vec3 in_Normal;

layout(location = 0) out vec4 out_Color;



vec2 nearest(vec2 pixel) {
	vec2 w = 0.5 * fwidth(pixel);
	return floor(pixel) + 0.5 + smoothstep(0.5 - w, 0.5 + w, fract(pixel));
}

vec3 vertexPosition(uint v) {
	return vec3(
		storage_PositionComponents[3 * v],
		storage_PositionComponents[3 * v + 1],
		storage_PositionComponents[3 * v + 2]
	);
}

vec3 vertexNormal(uint v) {
	return vec3(
		storage_NormalComponents[3 * v],
		storage_NormalComponents[3 * v + 1],
		storage_NormalComponents[3 * v + 2]
	);
}

vec3 interpolate(vec3 a, vec3 b, vec3 c, vec3 baryCoord) {
	return baryCoord.x * a + baryCoord.y * b + baryCoord.z * c;
}



void main() {
	vec2 size = textureSize(texture_FaceMap, 0);
	vec2 texCoord = nearest(in_TexCoord * size) / size;
	uint face = texture(texture_FaceMap, texCoord).r;
	if (face == ~0) {
		face = gl_PrimitiveID;
	}
	uvec3 vertices = uvec3(
		storage_Indices[3 * face],
		storage_Indices[3 * face + 1],
		storage_Indices[3 * face + 2]
	);
	vec2 texelBaryCoordXY =
		storage_Solvers[face].matrix *
		(texCoord - storage_Solvers[face].texCoord);
	vec3 texelBaryCoord = vec3(texelBaryCoordXY, 1.0 - texelBaryCoordXY.x - texelBaryCoordXY.y);

	vec4 viewPosition = uniform_View * vec4(interpolate(
		vertexPosition(vertices[0]),
		vertexPosition(vertices[1]),
		vertexPosition(vertices[2]),
		texelBaryCoord
	), 1.0);
	vec3 position = viewPosition.xyz / viewPosition.w;
	vec3 normal = normalize(mat3(uniform_View) * interpolate(
		vertexNormal(vertices[0]),
		vertexNormal(vertices[1]),
		vertexNormal(vertices[2]),
		texelBaryCoord
	));

	vec3 albedo = texture(texture_Albedo, texCoord).rgb;

	const float gloss = 0.5;
	const float glossPower = 32.0;
	const vec3 ambient = vec3(0.25);
	const vec3 directColor = vec3(0.75);
	const vec3 direction = normalize(vec3(0.0, 1.0, 1.0));

	float diffuseIntensity = max(dot(normal, direction), 0.0);
	vec3 diffuse = directColor * diffuseIntensity;

	vec3 halfway = normalize(direction - normalize(position));
	float specularIntensity = pow(max(dot(normal, halfway), 0.0), glossPower);
	vec3 specular = gloss * directColor * specularIntensity;

	out_Color = vec4(albedo * (ambient + diffuse + specular), 1.0);
}
