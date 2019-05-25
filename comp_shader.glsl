#version 440 core
layout( local_size_x = 16, local_size_y = 16) in;

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

struct Plane
{
	vec3 normal;
	vec3 point;

	int material_id;
};

struct Triangle
{
	vec3 vertices[3];

	int material_id;
};

Raytrace_result TraceWithSphere(Ray ray, Sphere sphere);
Raytrace_result TraceWithPlane(Ray ray, Plane plane);
Raytrace_result TraceWithTriangle(Ray ray, Triangle triangle);

#define REFLECTIONS 7

//************primitives**************************************

uniform int triangles_amount;
uniform int spheres_amount;
uniform int planes_amount;


Sphere[] spheres = 
{
	{vec3(0, 0, 0), 1, 6},
};

Plane planes[] = 
{
	{normalize(vec3(0, 1, 0)), vec3(0, -3, 0), 3},//bottom
	{normalize(vec3(0, -1, 0)), vec3(0, 3, 0), 6},//top

	{normalize(vec3(-1, 0, 0)), vec3(3, 0, 0), 0},//right
	{normalize(vec3(1, 0, 0)), vec3(-3, 0, 0), 1},//left

	{normalize(vec3(0, 0, 1)), vec3(0, 0, -3), 3},//far
	{normalize(vec3(0, 0, -1)), vec3(0, 0, 3), 3}//near
};



//******************triangles*******************
layout(std430, binding = 0) buffer tr_vertices//3 vertices per triangle
{
	vec4 triangle_vertices[];
};

layout(std430, binding = 1) buffer tr_materials
{
	int triangle_materials[];
};

struct AABB
{
	vec3 _min;
	vec3 _max;
};

struct kDtree_node
{
	int left;
	int right;
	int parent;

	int index;
};

struct kDtree_leaf
{
	int parent;

	int index;

	int triangle_insdexes_pos;
	int triangle_insdexes_length;
};

uniform int node_count;

layout(std430, binding = 2) buffer _nodes
{
	kDtree_node[] nodes;
};

layout(std430, binding = 3) buffer _leaves
{
	kDtree_leaf[] leaves;
};

layout(std430, binding = 4) buffer _triangle_indexes_kDtree
{
	int[] triangle_indexes_kDtree;
};

layout(std430, binding = 5) buffer _aabbs
{
	vec4[] aabbs;//by index
};

AABB getAABBbyIndex(int index)
{
	return AABB(aabbs[index * 2].xyz, aabbs[index * 2 + 1].xyz);
}

//********************spheres*******************
layout(std430, binding = 6) buffer sp_centers
{
	vec4 sphere_centers[];
};

layout(std430, binding = 7) buffer sp_radiuses
{
	float sphere_radiuses[];
};

layout(std430, binding = 8) buffer sp_materials
{
	int sphere_materials[];
};
//*******************planes***********************
layout(std430, binding = 9) buffer pl_points
{
	vec4 plane_points[];
};

layout(std430, binding = 10) buffer pl_normals
{
	vec4 plane_normals[];
};

layout(std430, binding = 11) buffer pl_materials
{
	int plane_materials[];
};
//************************************************


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
	{vec3(0, 0, 1), vec3(0, 0, 0),	0.0,		0.5,	0.0,		1.00},//2
	{vec3(0.5),     vec3(0, 0, 0),	0.0,		0.0,	0.0,		1.00},//3
	{vec3(0.5),		vec3(0, 0, 0),	0.0,		0.0,	1.0,		1.20},//4
	{vec3(1, 1, 1), vec3(0, 0, 0),	0.0,		0.0,	0.0,		1.00},//5
	{vec3(0, 0, 0), vec3(2, 2, 2),	1.0,		0.0,	0.0,		1.00},//6
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

	vec3 normal = normalize(cross(triangle.vertices[0].xyz - triangle.vertices[1].xyz, triangle.vertices[0].xyz - triangle.vertices[2].xyz));
	Plane triangle_plane = {normal, triangle.vertices[0].xyz, 0};
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
		vec3 P_normal = cross(triangle.vertices[j].xyz - triangle.vertices[k].xyz, triangle_plane.normal);
		//plane equality is P_normal.x * X + P_normal.y * Y + P_normal.z * Z + d = 0 (normal can be unnormalized)
		float d = -dot(P_normal, triangle.vertices[j].xyz);

		float I_side = sign(dot(P_normal, triangle.vertices[i].xyz) + d);
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

Raytrace_result TraceWithBox(Ray ray, AABB box)
{
	Raytrace_result result;
	result.intersection = false;

	float tmin, tmax, tymin, tymax, tzmin, tzmax;
	
	if(EqualsZero(ray.direction.x))
	{
		tmin = -INFINITY;
		tmax = INFINITY;
	}
	else if (ray.direction.x > 0) 
	{
		tmin = (box._min.x - ray.source.x) / ray.direction.x;
		tmax = (box._max.x - ray.source.x) / ray.direction.x;
	}	
	else
	{
		tmin = (box._max.x - ray.source.x) / ray.direction.x;
		tmax = (box._min.x - ray.source.x) / ray.direction.x;
	}

	if(EqualsZero(ray.direction.y))
	{
		tymin = -INFINITY;
		tymax = INFINITY;
	}
	else if (ray.direction.y > 0) {
		tymin = (box._min.y - ray.source.y) / ray.direction.y;
		tymax = (box._max.y - ray.source.y) / ray.direction.y;
	}
	else {
		tymin = (box._max.y - ray.source.y) / ray.direction.y;
		tymax = (box._min.y - ray.source.y) / ray.direction.y;
	}

	if ( (tmin > tymax) || (tymin > tmax) )
		return result;
	if (tymin > tmin)
		tmin = tymin;
	if (tymax < tmax)
		tmax = tymax;

	if(EqualsZero(ray.direction.z))
	{
		tzmin = -INFINITY;
		tzmax = INFINITY;
	}
	else if (ray.direction.z > 0) 
	{
		tzmin = (box._min.z - ray.source.z) / ray.direction.z;
		tzmax = (box._max.z - ray.source.z) / ray.direction.z;
	}
	else 
	{
		tzmin = (box._max.z - ray.source.z) / ray.direction.z;
		tzmax = (box._min.z - ray.source.z) / ray.direction.z;
	}
	if ( (tmin > tzmax) || (tzmin > tmax) )
		return result;
	if (tzmin > tmin)
		tmin = tzmin;
	if (tzmax < tmax)
		tmax = tzmax;

	if(tmin < ray.max_value && tmin > ray.min_value)
	{
		result.t = tmin;
		result.intersection = true;
		result.contact = ray.source + ray.direction * tmin;
	}
	else if(tmax < ray.max_value && tmax > ray.min_value)
	{
		result.t = tmax;
		result.intersection = true;
		result.contact = ray.source + ray.direction * tmax;
	}

	return result;
}


//************************************************************
bool[1024] KdTreeAllChecked;

Raytrace_result TraceInKdTree(Ray ray, float curr_t)
{
	int curr_node = 0;
	int prev_node = -1;

	if(!TraceWithBox(ray, getAABBbyIndex(0)).intersection)
	{
		Raytrace_result res;
		res.intersection = false;
		return res;
	}

	for(int i = 0; i < KdTreeAllChecked.length(); i++)
		KdTreeAllChecked[i] = false;

	while(true)
	{
		if(curr_node > prev_node)//moving down
		{
			if(curr_node < node_count)//node
			{
				Raytrace_result withleft = TraceWithBox(ray, getAABBbyIndex(nodes[curr_node].left));
				Raytrace_result withright = TraceWithBox(ray, getAABBbyIndex(nodes[curr_node].right));

				if(withleft.intersection && withright.intersection)
				{
					if(withleft.t < withright.t)
					{//left first
						prev_node = curr_node;
						curr_node = nodes[curr_node].left;
					}
					else
					{//right first
						prev_node = curr_node;
						curr_node = nodes[curr_node].right;
					}
				}
				else if(withleft.intersection)
				{
					prev_node = curr_node;
					curr_node = nodes[curr_node].left;
				}
				else if(withright.intersection)
				{
					prev_node = curr_node;
					curr_node = nodes[curr_node].right;
				}			
			}
			else//leaf
			{
				Raytrace_result best_res;
				best_res.t = curr_t;
				best_res.intersection = false;
				int triangle_index;

				//if(curr_t > TraceWithBox(ray, getAABBbyIndex(curr_node)).t)
				{
					kDtree_leaf curr_leaf = leaves[curr_node - node_count];

					for(int i = curr_leaf.triangle_insdexes_pos; i < curr_leaf.triangle_insdexes_pos + curr_leaf.triangle_insdexes_length; i++)
					{
						int tr_index = triangle_indexes_kDtree[i];

						Raytrace_result res = TraceWithTriangle(ray, 
						Triangle(vec3[](triangle_vertices[3 * tr_index].xyz, triangle_vertices[3 * tr_index + 1].xyz, triangle_vertices[3 * tr_index + 2].xyz), 0));

						if(res.intersection && res.t <= best_res.t)
						{
							best_res = res;
							triangle_index = tr_index;
						}
					}
				}

				if(best_res.intersection)
				{
					best_res.material_id = triangle_materials[triangle_index];
					return best_res;
				}

				prev_node = curr_node;
				curr_node = leaves[curr_node - node_count].parent;
			}
		}
		else//moving up
		{
			if(KdTreeAllChecked[curr_node])
			{//move up
				prev_node = curr_node;
				curr_node = nodes[curr_node].parent;

				if(curr_node < 0)//reached root
				{
					Raytrace_result res;
					res.intersection = false;
					return res;
				}

				continue;
			 }

			 KdTreeAllChecked[curr_node] = true;

			if(nodes[curr_node].left == prev_node)//lifting from left
			{
				if(TraceWithBox(ray, getAABBbyIndex(nodes[curr_node].right)).intersection)
				{
					prev_node = curr_node;
					curr_node = nodes[curr_node].right;
				}
				else
				{
					prev_node = curr_node;
					curr_node = nodes[curr_node].parent;

					if(curr_node < 0)//reached root
					{
						Raytrace_result res;
						res.intersection = false;
						return res;
					}
				}
			}
			else//lifting from right
			{
				if(TraceWithBox(ray, getAABBbyIndex(nodes[curr_node].left)).intersection)
				{
					prev_node = curr_node;
					curr_node = nodes[curr_node].left;
				}
				else
				{
					prev_node = curr_node;
					curr_node = nodes[curr_node].parent;

					if(curr_node < 0)//reached root
					{
						Raytrace_result res;
						res.intersection = false;
						return res;
					}
				}
			}
		}

	}
}

Raytrace_result TraceRay(Ray ray)
{
	Raytrace_result result;
	result.t = INFINITY;
	result.intersection = false;

	
	for(int i = 0; i < spheres_amount; i++)//find sphere with min t
	{
		Raytrace_result res = TraceWithSphere(ray, spheres[i]);//Sphere(sphere_centers[i].xyz, sphere_radiuses[i], sphere_materials[i]));

		if(res.intersection && res.t < result.t)
		{
			result = res;
			result.material_id = spheres[i].material_id;//sphere_materials[i];
		}
	}
	

	
	for(int i = 0; i < planes_amount; i++)//find plane with min t
	{
		Raytrace_result res = TraceWithPlane(ray, planes[i]);//Plane(plane_normals[i].xyz, plane_points[i].xyz, plane_materials[i]));

		if(res.intersection && res.t < result.t)
		{
			result = res;
			result.material_id = planes[i].material_id;//plane_materials[i];
		}
	}

	Raytrace_result res = TraceInKdTree(ray, result.t);

	if(res.intersection)
		result = res;
	

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
		// from refl + refr + emissive to 1 is diffuse
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
			 
			vec3 new_direction = refract(current_ray.direction, result.normal * result.normal_facing_outside, relative_refraction);
		
			if(EqualsZero(length(new_direction)))
			 current_ray.direction = normalize(cross(cross(current_ray.direction, result.normal), result.normal));
			 else
			 current_ray.direction = new_direction;
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

	vec3 color;
	color = GetColor(current_ray);

	imageStore(Texture, ivec2(gl_GlobalInvocationID.xy), vec4((color + imageLoad(Texture, ivec2(gl_GlobalInvocationID.xy)).xyz * (iteration - 1)) / iteration, 1));
}