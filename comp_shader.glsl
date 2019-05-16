#version 430 core
layout( local_size_x = 32, local_size_y = 32) in;

layout (binding = 0, rgba8) uniform image2D Texture;

#define ZERO 0.00001
#define INFINITY 1000000

bool EqualsZero(float a)
{
	return ((a > -ZERO) && (a < ZERO));
}

uniform float rand_seed;

float Rand(vec2 point)
{
	return fract(sin(dot(point, vec2(0.7685, rand_seed))) * 43758.5453123);
}

struct Ray
{
	vec3 source;
	vec3 direction;

	float min_value;
	float max_value;
};

struct Raytrace_result
{
	bool intersection;
	vec3 contact;
	vec3 normal;
	//1 if yes -1 if no
	float normal_facing_outside;
	float t;

	int material_id;
};

struct Sphere
{
	vec3 center;
	float radius;

	int material_id;
};

struct Triangle
{
	vec3[3] vertices;

	int material_id;
};

struct Plane
{
	vec3 normal;
	vec3 point;

	int material_id;
};

Raytrace_result TraceWithSphere(Ray ray, Sphere sphere);
Raytrace_result TraceWithPlane(Ray ray, Plane plane);
Raytrace_result TraceWithTriangle(Ray ray, Triangle triangle);

#define REFLECTIONS 10

//************primitives**************************************

#define SPHERES_COUNT 4
#define PLANES_COUNT 6
#define TRIANGLES_COUNT 0

#if SPHERES_COUNT != 0
Sphere[SPHERES_COUNT] spheres = 
{
	{vec3(1, 0, -2), 0.5, 3},
	{vec3(-1, 0, -2), 0.5, 3},
	{vec3(1, -1, -2), 0.5, 3},
	{vec3(-1, -1, -2), 0.5, 3},
};
#endif

#if PLANES_COUNT != 0
Plane[PLANES_COUNT] planes = 
{//normal point material_id
	{normalize(vec3(0, 1, 0)), vec3(0, -3, 0), 3},//bottom
	{normalize(vec3(0, -1, 0)), vec3(0, 3, 0), 6},//top

	{normalize(vec3(-1, 0, 0)), vec3(3, 0, 0), 0},//right
	{normalize(vec3(1, 0, 0)), vec3(-3, 0, 0), 1},//left

	{normalize(vec3(0, 0, 1)), vec3(0, 0, -3), 3},//far
	{normalize(vec3(0, 0, -1)), vec3(0, 0, 3), 3}//near
};
#endif

#if TRIANGLES_COUNT != 0
Triangle[TRIANGLES_COUNT] triangles = 
{//vertices[3] material_id
	{{vec3(-1, -1, 1),			vec3(0, -1, 1 - sqrt(3)),	vec3(1, -1, 1)}, 4},
	{{vec3(-1, -1, 1),			vec3(1, -1, 1),				vec3(0, 1, 1 - 1 / sqrt(3))}, 4},
	{{vec3(-1, -1, 1),			vec3(0, 1, 1 - 1 / sqrt(3)),vec3(0, -1, 1 - sqrt(3))}, 4},
	{{vec3(0, 1, 1 - 1 / sqrt(3)),vec3(1, -1, 1),			vec3(0, -1, 1 - sqrt(3))}, 4}
};
#endif
//******************materials*********************************

struct Material
{
	vec3 color;
	vec3 emission;
	float emissive;
	float reflective;

	float refractive;
	float refraction;
};

Material[] materials = 
{//  color          emission		emissive reflective refractive refraction
	{vec3(1, 0, 0), vec3(1, 1, 1),	0.0,		0.0,	0.0,		1.00},//0
	{vec3(0, 1, 0), vec3(1, 1, 1),	0.0,		0.0,	0.0,		1.00},//1
	{vec3(0.5),     vec3(0, 0, 0),	0.0,		0.5,	0.0,		1.00},//2
	{vec3(0.5),     vec3(0, 0, 0),	0.0,		0.0,	0.0,		1.00},//3
	{vec3(0.5),		vec3(0, 0, 0),	0.0,		0.0,	1.0,		1.20},//4
	{vec3(0, 0, 1), vec3(0, 0, 0),	0.0,		0.0,	0.0,		1.00},//5
	{vec3(0, 0, 1), vec3(2, 2, 2),	1.0,		0.0,	0.0,		1.00},//6
};
//************************camera******************************
vec3 view_point = vec3(0, 0, 10);
float view_distance = 7.01;
uniform mat3 rotation_mat;
vec2 viewport = vec2(5.99, 5.99);

uniform vec2 resolution;
//************************************************************

Raytrace_result TraceWithSphere(Ray ray, Sphere sphere)
{
	Raytrace_result result;

	vec3 A = sphere.center - ray.source;
	//length(Direction * t + Source - Center) = radius
	// A = center - source
	//t^2 * dot(Direction, Direction) - 2 * t * dot(A, Direction) + dot(A, A) = Radius ^ 2
	//Direction is normalized => dot(Direction, Direction) = 1
	float half_second_k = -dot(A, ray.direction);
	//Discriminant = second_k ^ 2 - 4 * first_k * third_k
	float Discriminant = 4 * (half_second_k * half_second_k - (dot(A, A) - sphere.radius * sphere.radius));

	if(Discriminant < 0)//no intersection
	{
		result.intersection = false;
		return result;
	}

	//roots are (-half_second_k * 2 +- sqrtD) / 2
	float sqrtD = sqrt(Discriminant);
	float t1 = -half_second_k + sqrtD / 2;
	float t2 = -half_second_k - sqrtD / 2;

	if(t2 >= ray.min_value && t2 <= ray.max_value)
	{
		result.t = t2;
		result.normal_facing_outside = 1;
	}
	else if(t1 >= ray.min_value && t1 <= ray.max_value)
	{
		result.t = t1;
		//if we choose max value of t it means that ray is traced from inside
		result.normal_facing_outside = -1;
	}
	else
	{
		result.intersection = false;
		return result;
	}
	
	result.contact = result.t * ray.direction + ray.source;
	result.normal = (result.contact - sphere.center) / sphere.radius;
	result.intersection = true;

	return result;
}

Raytrace_result TraceWithPlane(Ray ray, Plane plane)
{
	Raytrace_result result;

	//plane equality:
	//Nx(x - x0) + Ny(y - y0) + Nz(z - z0) = 0
	//where N - normal vector to plane
	//V[0](x0, y0, z0) - any point on this plane
	//point on ray = t * Direction + source
	//   =>
	//t = Dot(N, V[0] - Source) / Dot(N, Direction)
	//Dot(N, Direction) == 0 when Normal is perpendicular to direction => Direction parrallel to plane
	float t = dot(plane.normal, plane.point - ray.source) / dot(plane.normal, ray.direction);
	
	if(t < ray.min_value || t > ray.max_value)//t is not valid
	{		
		result.intersection = false;
		return result;
	}

	result.intersection = true;
	result.contact = ray.source + ray.direction * t;		
	result.normal = plane.normal;
	result.normal_facing_outside = sign(dot(plane.normal, -ray.direction));
	result.t = t;

	return result;
}
 
Raytrace_result TraceWithTriangle(Ray ray, Triangle triangle)
{
	Raytrace_result result;

	vec3 normal = normalize(cross(triangle.vertices[0] - triangle.vertices[1], triangle.vertices[0] - triangle.vertices[2]));
	Plane triangle_plane = {normal, triangle.vertices[0], 0};
	result = TraceWithPlane(ray, triangle_plane);

	if(!result.intersection)
	{
		return result;
	}

	for(int i = 0; i < 3; i++)
	{
		int j = int(mod(i + 1, 3));//second vertex
		int k = int(mod(i + 2, 3));//third vertex

		//determine plane P that is parallel to triangle normal & contains JK
		vec3 P_normal = cross(triangle.vertices[j] - triangle.vertices[k], triangle_plane.normal);
		//plane equality is P_normal.x * X + P_normal.y * Y + P_normal.z * Z + d = 0 (normal can be unnormalized)
		float d = -dot(P_normal, triangle.vertices[j]);

		float I_side = sign(dot(P_normal, triangle.vertices[i]) + d);
		float Contact_side = sign(dot(P_normal, result.contact) + d);

		if(Contact_side == 0)
			continue;

		if(I_side == Contact_side)
			continue;

		result.intersection = false;
		return result;
	}

	return result;
}
//************************************************************

Raytrace_result TraceRay(Ray ray)
{
	float min_t = INFINITY;

	Raytrace_result result;
	result.intersection = false;

	#if SPHERES_COUNT != 0
	for(int i = 0; i < SPHERES_COUNT; i++)//find sphere with min t
	{
		Raytrace_result res = TraceWithSphere(ray, spheres[i]);

		if(res.intersection && res.t < min_t)
		{
			min_t = res.t;
			result = res;
			result.material_id = spheres[i].material_id;
		}
	}
	#endif

	#if PLANES_COUNT != 0
	for(int i = 0; i < PLANES_COUNT; i++)//find plane with min t
	{
		Raytrace_result res = TraceWithPlane(ray, planes[i]);

		if(res.intersection && res.t < min_t)
		{
			min_t = res.t;
			result = res;
			result.material_id = planes[i].material_id;
		}
	}
	#endif

	#if TRIANGLES_COUNT != 0
	for(int i = 0; i < TRIANGLES_COUNT; i++)//find triangle with min t
	{
		Raytrace_result res = TraceWithTriangle(ray, triangles[i]);

		if(res.intersection && res.t < min_t)
		{
			min_t = res.t;
			result = res;
			result.material_id = triangles[i].material_id;
		}
	}
	#endif

	return result;
}

//************************************************************
vec3 GetColor(Ray ray)
{
	vec3 color = vec3(1);

	Ray current_ray = ray;

	for(int i = 0; i < REFLECTIONS; i++)
	{
		Raytrace_result result = TraceRay(current_ray);	

		if(!result.intersection)
		{
			return vec3(0);
		}

		Material material = materials[result.material_id];		

		current_ray.source = result.contact;
		current_ray.min_value = ZERO;
		current_ray.max_value = INFINITY;

		// generating random
		// from 0 to refl is reflection
		// from refl to refr + refl is refraction
		// from refl + refr to refl + refr + emissive is emission
		//refl + refr + emissive to 1 is diffuse
		float rand = Rand(gl_GlobalInvocationID.xy * 10);
		
		if(rand < material.reflective)
		{//reflection
			current_ray.direction = reflect(current_ray.direction, result.normal);
		}
		else if(rand < material.reflective + material.refractive)
		{//refraction
			float a_c = dot(result.normal * result.normal_facing_outside, current_ray.direction);
			float a_s = sqrt(1 - a_c * a_c);
			
			//if facing outside then 1 / mat.refraction else mat.refraction
			float relative_refraction = pow(material.refraction, -result.normal_facing_outside);
			 
			current_ray.direction = refract(current_ray.direction, result.normal * result.normal_facing_outside, relative_refraction);
		}
		else if(rand < material.reflective + material.refractive + material.emissive)
		{//emission
			return color * material.emission;
		}
		else	
		{//diffuse
			//choosing random direction in hemisphere
			vec3 rand_direction = normalize(vec3(Rand(gl_GlobalInvocationID.xy) * 2 - 1, Rand(gl_GlobalInvocationID.yx) * 2 - 1, Rand(gl_GlobalInvocationID.xy * 4) * 2 - 1));

			float a = 0.01;
			for(int i = 0; i < 10; i++)
			{
				if(dot(rand_direction, result.normal) < 0)//if not lies in hemisphere
				{
					rand_direction = -normalize(vec3(Rand(gl_GlobalInvocationID.xy * a / 3) * 2 - 1, Rand(gl_GlobalInvocationID.yx * a * 2) * 2 - 1, Rand(gl_GlobalInvocationID.xy + ivec2(a)) * 2 - 1));//*= -1;
					a += 1;
				}
			}
			//***************************************

			current_ray.direction = rand_direction;

			color *= 2 * dot(current_ray.direction, result.normal) * material.color;
		}		
		
	}

	return vec3(0);//too many bounces
}

//***********************************************************
uniform int iteration;

void main()
{ 
	Ray current_ray;
	current_ray.source = view_point;	

	vec3 watch_dot = view_point;
	watch_dot.z -= view_distance;//forward z
	watch_dot.x += ((gl_GlobalInvocationID.x / resolution.x) - 0.5) * viewport.x;
	watch_dot.y += ((gl_GlobalInvocationID.y / resolution.y) - 0.5) * viewport.y;

	current_ray.direction = normalize(watch_dot - view_point);
	current_ray.min_value = length(watch_dot - view_point);
	current_ray.max_value = INFINITY;

	current_ray.direction *= rotation_mat;

	vec3 color = GetColor(current_ray);

	imageStore(Texture, ivec2(gl_GlobalInvocationID.xy), vec4((color + imageLoad(Texture, ivec2(gl_GlobalInvocationID.xy)).xyz * (iteration - 1)) / iteration, 1));
}