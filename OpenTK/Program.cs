using System;
using OpenTK;
using OpenTK.Input;
using OpenTK.Graphics;
using OpenTK.Graphics.OpenGL4;
using System.IO;
using System.Drawing;
using System.Drawing.Imaging;
using PixelFormat = OpenTK.Graphics.OpenGL4.PixelFormat;
using System.Collections.Generic;

namespace Path_Tracing
{
    class Game : GameWindow
    {
        [STAThread]
        static void Main()
        {
            Game game = new Game();
            game.Run(60);
        }

        static int window_width = 512 * 2;
        static int window_height = 512 * 2;
        static int image_width = 512 * 2; 
        static int image_height = 512 * 2;
        static int workgroup_size = 32;//max 32
        public Game() : base(window_width, window_height, new GraphicsMode(new ColorFormat(8, 8, 8, 0), 24, 8, 4/*msaa*/, new ColorFormat(8, 8, 8, 0), 2), "PathTracing")
        {
            VSync = VSyncMode.On;
        }

        protected override void OnResize(EventArgs E)
        {
            base.OnResize(E);
            GL.Viewport(ClientRectangle.X, ClientRectangle.Y, ClientRectangle.Width, ClientRectangle.Height);
        }

        int compute_shader;
        int render_shader;
        int VAO, VBO;
        int texture;

        protected override void OnLoad(EventArgs E)
        {
            base.OnLoad(E);

            float[] quad_vertices =
            {
                -1, -1,  0, 0,
                -1, 1,   0, 1,
                1, 1,    1, 1,
                1, -1,   1, 0
            };

            VAO = GL.GenVertexArray();
            VBO = GL.GenBuffer();

            GL.BindVertexArray(VAO);
            {
                GL.BindBuffer(BufferTarget.ArrayBuffer, VBO);
                GL.BufferData(BufferTarget.ArrayBuffer, quad_vertices.Length * sizeof(float), quad_vertices, BufferUsageHint.StaticDraw);

                GL.VertexAttribPointer(0, 2, VertexAttribPointerType.Float, false, 4 * sizeof(float), 0);
                GL.EnableVertexAttribArray(0);
                GL.VertexAttribPointer(1, 2, VertexAttribPointerType.Float, false, 4 * sizeof(float), 2 * sizeof(float));
                GL.EnableVertexAttribArray(1);

                GL.BindBuffer(BufferTarget.ArrayBuffer, 0);
            }
            GL.BindVertexArray(0);

            GL.PolygonMode(MaterialFace.FrontAndBack, PolygonMode.Fill);

            render_shader = CompileShaders.Compile(new StreamReader("frag_shader.glsl"), new StreamReader("vert_shader.glsl"));
            compute_shader = CompileShaders.CompileComputeShader(new StreamReader("comp_shader.glsl"));

            GL.UseProgram(compute_shader);

            texture = GL.GenTexture();
            GL.BindTexture(TextureTarget.Texture2D, texture);
            GL.TexStorage2D(TextureTarget2d.Texture2D, 1, SizedInternalFormat.Rgba8, image_width, image_height);
            GL.TextureParameter(texture, TextureParameterName.TextureMinFilter, (int)All.Linear);
            GL.TextureParameter(texture, TextureParameterName.TextureMagFilter, (int)All.Linear);
            GL.BindImageTexture(0, texture, 0, false, 0, TextureAccess.ReadWrite, SizedInternalFormat.Rgba8);

            //*******************camera setup*******************
            GL.Uniform2(GL.GetUniformLocation(compute_shader, "resolution"), new Vector2(image_width, image_height));
            Matrix3 rotation_matrix = Matrix3.Identity;
            GL.UniformMatrix3(GL.GetUniformLocation(compute_shader, "rotation_mat"), false, ref rotation_matrix);
            GL.Uniform3(GL.GetUniformLocation(compute_shader, "view_point"), new Vector3(0, 0, 10));
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "view_distance"), 7.01f);
            GL.Uniform2(GL.GetUniformLocation(compute_shader, "viewport"), new Vector2(5.99f, 5.99f));
            //*************************************************
            #region spheres
            Sphere[] spheres = new Sphere[]
            {
                new Sphere() {center = new Vector3(3, -3, 1), radius = 1, material = 0 },
                new Sphere() {center = new Vector3(0, -1, 0), radius = 1, material = 4 },
            };

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "spheres_amount"), spheres.Length);
            int spheres_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 0, spheres_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, spheres.Length * Sphere.size, spheres, BufferUsageHint.StaticDraw);
            #endregion

            #region planes
            Plane[] planes = new Plane[]
            {
                new Plane() {normal = new Vector3(0, -1, 0).Normalized(), point = new Vector3(0, 3, 0), material_id = 0},//top
                
                new Plane() {normal = new Vector3(0, 1, 0).Normalized(), point = new Vector3(0, -3, 0), material_id = 1},//bottom
                
                new Plane() {normal = new Vector3(-1, 0, 0).Normalized(), point = new Vector3(3, 0, 0), material_id = 2},//right
                new Plane() {normal = new Vector3(1, 0, 0).Normalized(), point = new Vector3(-3, 0, 0), material_id = 3},//left

                new Plane() {normal = new Vector3(0, 0, 1).Normalized(), point = new Vector3(0, 0, -3), material_id = 1},//far
                new Plane() {normal = new Vector3(0, 0, -1).Normalized(), point = new Vector3(0, 0, 3), material_id = 1},//near
            };

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "planes_amount"), planes.Length);
            int planes_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 1, planes_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, planes.Length * Plane.size, planes, BufferUsageHint.StaticDraw);
            #endregion

            #region materials
            Material[] materials = new Material[]
            {
                new Material() {color = new Vector3(1.0f, 1.0f, 1.0f), emission = new Vector3(1, 1, 1), emmisive = 1, reflective = 0},//0
                new Material() {color = new Vector3(1.0f, 1.0f, 1.0f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 0},//1
                new Material() {color = new Vector3(1.0f, 0.2f, 0.2f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 0},//2
                new Material() {color = new Vector3(0.2f, 0.2f, 1.0f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 0},//3
                new Material() {color = new Vector3(0.2f, 0.2f, 1.0f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 1},//4
            };

            int materials_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 2, materials_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, materials.Length * Material.size, materials, BufferUsageHint.StaticDraw);
            #endregion

            //LoadTrianglesToBuffers(modelPath + ".obj", 14, 2);
            GC.Collect(); // buildKdTree and LoadObj
        }

        public struct Triangle
        {
            public Vector3[] vertices;
        }

        struct Sphere
        {
            public Vector3 center;
            public float radius;
            public int material;

            public float pad0;
            public float pad1;
            public float pad2;

            public static int size = 8 * sizeof(float);
        }

        struct Plane
        {
            public Vector3 normal;
            public float pad_0;
            public Vector3 point;
            public int material_id;

            public static int size = 8 * sizeof(float);
        }

        struct Material
        {
            public Vector3 color;
            public float reflective;
            public Vector3 emission;
            public float emmisive;
            public float refractive;
            public float refraction;

            public float pad0;
            public float pad1;

            public static int size = 12 * sizeof(float);
        }

        static string modelPath = "stanford-dragon";

        void LoadTrianglesToBuffers(string Model_Path, int max_tree_depth, int material_id)
        {
            GL.UseProgram(compute_shader);

            LoadObj loadObj = new LoadObj();
            BuildKDTree buildKdTree = new BuildKDTree();

            loadObj.Load(new StreamReader(Model_Path));
            Triangle[] triangles = loadObj.triangles.ToArray();

            try
            {
                buildKdTree.LoadFromJson(new StreamReader(modelPath + ".tree"));
                Console.WriteLine("cached tree found");
            }
            catch
            {
                Console.WriteLine("building tree...");
                buildKdTree.Build(triangles, max_tree_depth);
                buildKdTree.CacheIntoJson(new StreamWriter(modelPath + ".tree"));
                Console.WriteLine("tree built");
            }

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "triangles_amount"), triangles.Length);

            int triangle_vertices = GL.GenBuffer(),
            triangle_materials = GL.GenBuffer();

            List<Vector3> vert_list = new List<Vector3>();
            foreach (Triangle tri in triangles)
            {
                vert_list.AddRange(tri.vertices);
            }
            Vector4[] vert_array = vert_list.ConvertAll((x) => new Vector4(x, 1)).ToArray();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 0, triangle_vertices);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(float) * 4 * vert_array.Length, vert_array, BufferUsageHint.StaticDraw);

            int[] materials = new int[triangles.Length];
            for (int i = 0; i < materials.Length; i++)
            { materials[i] = material_id; }

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 1, triangle_materials);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(int) * materials.Length, materials, BufferUsageHint.StaticDraw);
            //*************triangle_tree*****************************
            int nodes = GL.GenBuffer(),
                leaves = GL.GenBuffer(),
                triangle_indexes = GL.GenBuffer(),
                aabbs = GL.GenBuffer();

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "node_count"), buildKdTree.preparedTreeData.nodes.Count);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 2, nodes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * buildKdTree.preparedTreeData.nodes.Count, buildKdTree.preparedTreeData.nodes.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 3, leaves);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * buildKdTree.preparedTreeData.leaves.Count, buildKdTree.preparedTreeData.leaves.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 4, triangle_indexes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * buildKdTree.preparedTreeData.triangle_indexes_tree.Count, buildKdTree.preparedTreeData.triangle_indexes_tree.ToArray(), BufferUsageHint.StaticDraw);

            List<Vector4> aabb_verts = new List<Vector4>();
            foreach (var aabb in buildKdTree.preparedTreeData.aabbs)
            {
                aabb_verts.Add(new Vector4(aabb.min_x, aabb.min_y, aabb.min_z, 1));
                aabb_verts.Add(new Vector4(aabb.max_x, aabb.max_y, aabb.max_z, 1));
            }
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 5, aabbs);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(float) * 8 * buildKdTree.preparedTreeData.aabbs.Count,
                aabb_verts.ToArray(), BufferUsageHint.StaticDraw);
        }

        int iterations = 1;
        Random rand = new Random();
        Vector2 offset = Vector2.Zero;

        void TracePath_Single()
        {
            GL.UseProgram(compute_shader);

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "iteration"), iterations);
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "rand_seed"), (rand.Next(1000000)) / 1000000f);
            GL.Uniform2(GL.GetUniformLocation(compute_shader, "offset"), offset);
            offset.X += workgroup_size;
            if(offset.X >= image_width)
            {
                offset.X = 0;
                offset.Y += workgroup_size;

                if (offset.Y >= image_height)
                {
                    offset.Y = 0;
                    Console.WriteLine(iterations);
                    iterations++;                    
                    
                    if (savebitmapflag)
                    {
                        SaveBitmap();
                        savebitmapflag = false;
                    }
                }
            }
            //************

            GL.MemoryBarrier(MemoryBarrierFlags.AllBarrierBits);
            GL.DispatchCompute(1, 1, 1);
            GL.MemoryBarrier(MemoryBarrierFlags.AllBarrierBits);     
        }

        protected override void OnRenderFrame(FrameEventArgs E)
        {
            base.OnRenderFrame(E);

            for(int i = 0; i < 1; i++)
            TracePath_Single();

            GL.BindVertexArray(VAO);
            {
                GL.UseProgram(render_shader);

                GL.DrawArrays(PrimitiveType.Quads, 0, 4);
            }
            GL.BindVertexArray(0);

            SwapBuffers();
        }

        protected override void OnKeyDown(KeyboardKeyEventArgs e)
        {
            base.OnKeyDown(e);

            if (e.Key == Key.Escape)
                Environment.Exit(1);

            if (e.Key == Key.S)
                savebitmapflag = true;
        }

        bool savebitmapflag = false;

        void SaveBitmap()
        {
            Bitmap bmp = new Bitmap(image_width, image_height);

            BitmapData data =
                bmp.LockBits(new Rectangle(0, 0, image_width, image_height), ImageLockMode.WriteOnly,
                System.Drawing.Imaging.PixelFormat.Format24bppRgb);

            GL.GetTexImage(TextureTarget.Texture2D, 0, PixelFormat.Bgr, PixelType.UnsignedByte, data.Scan0);
            bmp.UnlockBits(data);
            bmp.RotateFlip(RotateFlipType.RotateNoneFlipY);

            //gamma correction
            Bitmap gamma_corrected_bitmap = new Bitmap(image_width, image_height);
            ImageAttributes attributes = new ImageAttributes();
            attributes.SetGamma(1 / 2.2f);
            Point[] points =
            {
                    new Point(0, 0),
                    new Point(image_width, 0),
                    new Point(0, image_height),
            };
            Graphics.FromImage(gamma_corrected_bitmap).DrawImage(bmp, points, new Rectangle(0, 0, image_width, image_height),
            GraphicsUnit.Pixel, attributes);
            //****************

            gamma_corrected_bitmap.Save(modelPath + ".bmp", ImageFormat.Bmp);
            Console.WriteLine("saved");

            gamma_corrected_bitmap.Dispose();
            bmp.Dispose();
        }
    }
}