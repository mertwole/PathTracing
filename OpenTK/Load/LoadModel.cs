using OpenTK;
using OpenTK.Graphics.OpenGL4;
using System;
using System.Collections.Generic;
using System.IO;

namespace PathTracing.Load
{
    public struct Triangle
    {
        public Vector3[] vertices;
    }

    public static class LoadModel
    {
        public static void Load(string model_path, int max_tree_depth, int material_id)
        {
            LoadObj loadObj = new LoadObj();
            BuildKDTree buildKdTree = new BuildKDTree();

            loadObj.Load(new StreamReader(model_path + ".obj"));
            Triangle[] triangles = loadObj.triangles.ToArray();

            try
            {
                buildKdTree.LoadFromJson(new StreamReader(model_path + ".tree"));
                Console.WriteLine("cached tree found");
            }
            catch
            {
                Console.WriteLine("building tree...");
                buildKdTree.Build(triangles, max_tree_depth);
                buildKdTree.CacheIntoJson(new StreamWriter(model_path + ".tree"));
                Console.WriteLine("tree built");
            }

            GL.Uniform1(GL.GetUniformLocation(Game.compute_shader, "triangles_amount"), triangles.Length);

            int triangle_vertices = GL.GenBuffer(),
            triangle_materials = GL.GenBuffer();

            List<Vector3> vert_list = new List<Vector3>();
            foreach (Triangle tri in triangles)
            {
                vert_list.AddRange(tri.vertices);
            }
            Vector4[] vert_array = vert_list.ConvertAll((x) => new Vector4(x, 1)).ToArray();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 3, triangle_vertices);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(float) * 4 * vert_array.Length, vert_array, BufferUsageHint.StaticDraw);

            int[] materials = new int[triangles.Length];
            for (int i = 0; i < materials.Length; i++)
            { materials[i] = material_id; }

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 4, triangle_materials);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(int) * materials.Length, materials, BufferUsageHint.StaticDraw);
            //*************triangle_tree*****************************
            int nodes = GL.GenBuffer(),
                leaves = GL.GenBuffer(),
                triangle_indexes = GL.GenBuffer(),
                aabbs = GL.GenBuffer();

            GL.Uniform1(GL.GetUniformLocation(Game.compute_shader, "node_count"), buildKdTree.preparedTreeData.nodes.Count);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 5, nodes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * buildKdTree.preparedTreeData.nodes.Count, buildKdTree.preparedTreeData.nodes.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 6, leaves);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * buildKdTree.preparedTreeData.leaves.Count, buildKdTree.preparedTreeData.leaves.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 7, triangle_indexes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * buildKdTree.preparedTreeData.triangle_indexes_tree.Count, buildKdTree.preparedTreeData.triangle_indexes_tree.ToArray(), BufferUsageHint.StaticDraw);

            List<Vector4> aabb_verts = new List<Vector4>();
            foreach (var aabb in buildKdTree.preparedTreeData.aabbs)
            {
                aabb_verts.Add(new Vector4(aabb.min_x, aabb.min_y, aabb.min_z, 1));
                aabb_verts.Add(new Vector4(aabb.max_x, aabb.max_y, aabb.max_z, 1));
            }
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 8, aabbs);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(float) * 8 * buildKdTree.preparedTreeData.aabbs.Count,
                aabb_verts.ToArray(), BufferUsageHint.StaticDraw);
        }
    }
}
