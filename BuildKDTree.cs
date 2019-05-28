﻿using System.Collections.Generic;
using OpenTK;
using static Path_Tracing.Game;

namespace Path_Tracing
{
    class BuildKDTree
    {
        const float ZERO = 0.000001f;

        static bool EqualsZero(float num)
        {
            return num < ZERO && num > -ZERO;
        }

        const int SAHsamples = 16;
        const int MaxTriangles = 8;

        public struct AABB
        {
            public Vector3 min;
            public Vector3 max;
        }

        class Node
        {
            public Node left;
            public Node right;
            public Node parent;

            public AABB bounding_box;

            public List<int> triangle_indices;

            public int global_id;
        }

        static Node root;
        static Triangle[] triangles;       

        public static void Build(Triangle[] tris, int depth)
        {
            triangles = tris;

            root = new Node();
            root.bounding_box = new AABB { min = new Vector3(float.PositiveInfinity), max = new Vector3(float.NegativeInfinity) + new Vector3(1) };
            foreach (var tri in triangles/*get bounding box for all triangles*/)
            {
                for (int i = 0; i < 3; i++)
                {
                    if (root.bounding_box.min.X > tri.vertices[i].X)
                        root.bounding_box.min.X = tri.vertices[i].X;
                    if (root.bounding_box.min.Y > tri.vertices[i].Y)
                        root.bounding_box.min.Y = tri.vertices[i].Y;
                    if (root.bounding_box.min.Z > tri.vertices[i].Z)
                        root.bounding_box.min.Z = tri.vertices[i].Z;

                    if (root.bounding_box.max.X < tri.vertices[i].X)
                        root.bounding_box.max.X = tri.vertices[i].X;
                    if (root.bounding_box.max.Y < tri.vertices[i].Y)
                        root.bounding_box.max.Y = tri.vertices[i].Y;
                    if (root.bounding_box.max.Z < tri.vertices[i].Z)
                        root.bounding_box.max.Z = tri.vertices[i].Z;
                }
            }
            root.bounding_box.min -= new Vector3(0.001f);
            root.bounding_box.max += new Vector3(0.001f);

            root.triangle_indices = new List<int>();
            for(int i = 0; i < triangles.Length; i++)
                root.triangle_indices.Add(i);

            Split(root, 0, depth);

            PrepareToTransfer();
        }
        
        #region prepare_to_transfer
        public struct Tree_node
        {
            public int left;
            public int right;
            public int parent;

            public int index;
        }

        public struct Tree_leaf
        {
            public int parent;

            public int index;

            public int triangle_insdexes_pos;
            public int triangle_insdexes_length;
        }

        public static List<Tree_node> Prepared_nodes;
        public static List<Tree_leaf> Prepared_leaves;
        public static List<int> triangle_indexes_tree;
        public static List<AABB> aabbs;

        static void PrepareToTransfer()
        {
            List<Node> nodes = new List<Node>();
            List<Node> leaves = new List<Node>();

            List<Node> curr_layer = new List<Node>();
            curr_layer.Add(root);
            while (true)
            {
                List<Node> new_layer = new List<Node>();

                for (int i = 0; i < curr_layer.Count; i++)
                {
                    if (curr_layer[i].left == null)//leaf
                    {
                        leaves.Add(curr_layer[i]);
                    }
                    else//node
                    {
                        nodes.Add(curr_layer[i]);

                        new_layer.Add(curr_layer[i].left);
                        new_layer.Add(curr_layer[i].right);
                    }
                }

                if (new_layer.Count == 0)//wasnt nodes
                    break;

                curr_layer = new_layer;
            }

            for(int i = 0; i < nodes.Count; i++)
                nodes[i].global_id = i;

            for (int i = 0; i < leaves.Count; i++)
                leaves[i].global_id = i + nodes.Count;

            Prepared_leaves = new List<Tree_leaf>(leaves.Count);
            Prepared_nodes = new List<Tree_node>(nodes.Count);

            Prepared_nodes.Add(new Tree_node() { index = 0, left = nodes[0].left.global_id, right = nodes[0].right.global_id, parent = -1 });//root

            triangle_indexes_tree = new List<int>(); 

            curr_layer = new List<Node>();
            curr_layer.Add(root);
            while (true)
            {
                List<Node> new_layer = new List<Node>();

                for (int i = 0; i < curr_layer.Count; i++)
                {
                    if (curr_layer[i].left != null)//node
                    {
                        new_layer.Add(curr_layer[i].left);
                        new_layer.Add(curr_layer[i].right);
                        
                        if (curr_layer[i].left.left != null)//left isnt leaf
                            Prepared_nodes.Add(new Tree_node()
                            {
                                index = curr_layer[i].left.global_id,
                                left = curr_layer[i].left.left.global_id,
                                right = curr_layer[i].left.right.global_id,
                                parent = curr_layer[i].global_id
                            });
                        else//left is leaf
                        {
                            int ind_pos = triangle_indexes_tree.Count;
                            triangle_indexes_tree.AddRange(curr_layer[i].left.triangle_indices);
                            int ind_length = curr_layer[i].left.triangle_indices.Count;

                            Prepared_leaves.Add(new Tree_leaf()
                            {
                                index = curr_layer[i].left.global_id,
                                parent = curr_layer[i].global_id,
                                triangle_insdexes_pos = ind_pos,
                                triangle_insdexes_length = ind_length
                            });
                        }

                        if (curr_layer[i].right.left != null)//right isnt leaf
                            Prepared_nodes.Add(new Tree_node()
                            {
                                index = curr_layer[i].right.global_id,
                                left = curr_layer[i].right.left.global_id,
                                right = curr_layer[i].right.right.global_id,
                                parent = curr_layer[i].global_id
                            });
                        else//right is leaf
                        {
                            int ind_pos = triangle_indexes_tree.Count;
                            triangle_indexes_tree.AddRange(curr_layer[i].right.triangle_indices);
                            int ind_length = curr_layer[i].right.triangle_indices.Count;

                            Prepared_leaves.Add(new Tree_leaf()
                            {
                                index = curr_layer[i].right.global_id,
                                parent = curr_layer[i].global_id,
                                triangle_insdexes_pos = ind_pos,
                                triangle_insdexes_length = ind_length
                            });
                        }
                    }
                }

                if (new_layer.Count == 0)//wasnt nodes
                    break;

                curr_layer = new_layer;
            }

            Prepared_leaves.Sort((x, y) => x.index > y.index ? 1 : -1);
            Prepared_nodes.Sort((x, y) => x.index > y.index ? 1 : -1);

            aabbs = new List<AABB>();
            for (int i = 0; i < Prepared_nodes.Count; i++)
                aabbs.Add(nodes[i].bounding_box);
            for (int i = 0; i < Prepared_leaves.Count; i++)
                aabbs.Add(leaves[i].bounding_box);
        }
        #endregion

        static void Split(Node root, int depth, int maxdepth)
        {
            if (depth > maxdepth - 2)
                return;

            root.left = new Node();
            root.right = new Node();
            root.parent = root;
            root.left.triangle_indices = new List<int>();
            root.right.triangle_indices = new List<int>();

            Vector3 split_plane_normal;
            Vector3 split_plane_position = Vector3.Zero;
            //********************optimal split plane***********************
            float min_sah = float.PositiveInfinity;

            Vector3 diagonal = root.bounding_box.max - root.bounding_box.min;        
            //picking largest dimension
            if(diagonal.X > diagonal.Y && diagonal.X > diagonal.Z)
            {/*X*/split_plane_normal = new Vector3(1, 0, 0);}
            else if(diagonal.Y > diagonal.Z)
            {/*Y*/split_plane_normal = new Vector3(0, 1, 0);}
            else
            {/*Z*/split_plane_normal = new Vector3(0, 0, 1);}


            for (int i = 1; i < SAHsamples; i++)
            {
                Vector3 position = new Vector3(root.bounding_box.min + (root.bounding_box.max - root.bounding_box.min) * (i / (float)SAHsamples));
                //other dimensions instread larger not matters

                float curr_sah = SAH(root, split_plane_normal, position);
                if (curr_sah < min_sah)
                {
                    split_plane_position = position;
                    min_sah = curr_sah;
                }
            }            

            //**************************************************************
            root.left.bounding_box = new AABB()
            {
                min = root.bounding_box.min,
                max = split_plane_normal * split_plane_position + (new Vector3(1) - split_plane_normal) * root.bounding_box.max
            };

            root.right.bounding_box  = new AABB()
            {
                min = split_plane_normal * split_plane_position + (new Vector3(1) - split_plane_normal) * root.bounding_box.min,
                max = root.bounding_box.max
            };

            for(int i = 0; i < root.triangle_indices.Count; i++)
            {
                if (TrianglevsAABB(triangles[root.triangle_indices[i]], root.right.bounding_box))
                    root.right.triangle_indices.Add(root.triangle_indices[i]);

                if (TrianglevsAABB(triangles[root.triangle_indices[i]], root.left.bounding_box))
                    root.left.triangle_indices.Add(root.triangle_indices[i]);
            }

            if(root.left.triangle_indices.Count > MaxTriangles)
                Split(root.left, depth + 1, maxdepth);
            if (root.right.triangle_indices.Count > MaxTriangles)
                Split(root.right, depth + 1, maxdepth);
        }

        static bool TrianglevsAABB(Triangle triangle, AABB aabb)
        {
            Vector3[] aabb_vertices = new Vector3[]
            {
                new Vector3(aabb.min),
                new Vector3(aabb.max.X, aabb.min.Y, aabb.min.Z),
                new Vector3(aabb.max.X, aabb.min.Y, aabb.max.Z),
                new Vector3(aabb.min.X, aabb.min.Y, aabb.max.Z),

                new Vector3(aabb.min.X, aabb.max.Y, aabb.max.Z),
                new Vector3(aabb.min.X, aabb.max.Y, aabb.min.Z),
                new Vector3(aabb.max.X, aabb.max.Y, aabb.min.Z),
                new Vector3(aabb.max)
            };
            //edges are:
            //0-1-2-3||4-5-6-7||1-2-7-6||2-3-4-7||0-3-4-5||0-1-6-5

            //plane equality is normal.x * x + normal.y * y + normal.z * z + d = 0
            Vector3 triangle_normal = Vector3.Cross(triangle.vertices[0] - triangle.vertices[1], triangle.vertices[0] - triangle.vertices[2]);
            float triangle_d = -Vector3.Dot(triangle_normal, triangle.vertices[0]);

            int box_sign = 0;
            bool intersection = false;
            for (int i = 0; i < 8; i++)
            {
                float i_vert_side = (Vector3.Dot(triangle_normal, aabb_vertices[i]) + triangle_d);

                if (EqualsZero(i_vert_side))
                    continue;

                if (box_sign == 0)
                {
                    box_sign = i_vert_side > 0 ? 1 : -1;
                    continue;
                }

                if ((i_vert_side > 0 ? 1 : -1) != box_sign)
                {
                    intersection = true;
                    break;
                }
            }

            if (!intersection)
                return false;

            bool CheckBoxSide(int[] points, int opposite_side_point)
            {
                Vector3 normal = Vector3.Cross(aabb_vertices[points[0]] - aabb_vertices[points[1]], aabb_vertices[points[0]] - aabb_vertices[points[2]]);
                float d = -Vector3.Dot(normal, aabb_vertices[points[0]]);

                int triangle_side = ((Vector3.Dot(normal, aabb_vertices[opposite_side_point]) + d) < 0) ? 1 : -1;
                for (int i = 0; i < 3; i++)
                {
                    float triangle_dot_side = Vector3.Dot(normal, triangle.vertices[i]) + d;
                    if (EqualsZero(triangle_dot_side))
                        continue;

                    int tr_dot_side = (triangle_dot_side > 0) ? 1 : -1;

                    if (triangle_side != tr_dot_side)//intersection
                        return true;
                }

                return false;//separating plane found
            }

            if (!CheckBoxSide(new int[] { 0, 1, 2 }, 4))
                return false;
            if (!CheckBoxSide(new int[] { 4, 5, 6 }, 0))
                return false;
            if (!CheckBoxSide(new int[] { 1, 2, 7 }, 0))
                return false;
            if (!CheckBoxSide(new int[] { 2, 3, 4 }, 0))
                return false;
            if (!CheckBoxSide(new int[] { 0, 3, 4 }, 7))
                return false;
            if (!CheckBoxSide(new int[] { 0, 1, 6 }, 7))
                return false;

            return true;
        }

        static float SAH(Node node, Vector3 split_plane_normal, Vector3 split_plane_position)
        {
            Vector3 diagonal = node.bounding_box.max - node.bounding_box.min;

            float left_ratio = Vector3.Dot(split_plane_position - node.bounding_box.min, split_plane_normal) / Vector3.Dot(diagonal, split_plane_normal);
            Vector3 left_multiplier = 
                new Vector3(split_plane_normal.X != 0 ? 1 : left_ratio, 
                split_plane_normal.Y != 0 ? 1 : left_ratio, 
                split_plane_normal.Y != 0 ? 1 : left_ratio);
            float left_half_surface = Vector3.Dot(new Vector3(diagonal.Y * diagonal.Z, diagonal.X * diagonal.Y, diagonal.X * diagonal.Z), left_multiplier);
            float right_ratio = 1 - left_ratio;
            Vector3 right_multiplier =
                new Vector3(split_plane_normal.X != 0 ? 1 : right_ratio,
                split_plane_normal.Y != 0 ? 1 : right_ratio,
                split_plane_normal.Y != 0 ? 1 : right_ratio);
            float right_half_surface = Vector3.Dot(new Vector3(diagonal.Y * diagonal.Z, diagonal.X * diagonal.Y, diagonal.X * diagonal.Z), right_multiplier);


            int left_triangles = 0, right_triangles = 0;

            AABB left_aabb = new AABB()
            {
                min = node.bounding_box.min,
                max = split_plane_normal * split_plane_position + (new Vector3(1) - split_plane_normal) * node.bounding_box.max
            };

            AABB right_aabb = new AABB()
            {
                min = split_plane_normal * split_plane_position + (new Vector3(1) - split_plane_normal) * node.bounding_box.min,
                max = node.bounding_box.max
            };

            foreach (var triangle in node.triangle_indices)
            {
                if(TrianglevsAABB(triangles[triangle], left_aabb))
                {
                    left_triangles++;
                }

                if (TrianglevsAABB(triangles[triangle], right_aabb))
                {
                    right_triangles++;
                }
            }

            return left_half_surface * left_triangles + right_half_surface * right_triangles;
        }
    }
}