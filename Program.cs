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
            game.Run(120);
        }

        static int window_width = 512;
        static int window_height = 512;
        static int image_width = 512;
        static int image_height = 512;
        static int workgroup_size = 32;//max 32
        public Game() : base(window_width, window_height, new GraphicsMode(new ColorFormat(8, 8, 8, 0), 24, 8, 1/*msaa*/, new ColorFormat(8, 8, 8, 0), 2), "PathTracing")
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
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "spheres_amount"), 0);
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "planes_amount"), 6);

            LoadTrianglesToBuffers(modelPath + ".obj", 10, 2);   
        }

        public struct Triangle
        {
            public Vector3[] vertices;
        }

        static string modelPath = "sphere";

        void LoadTrianglesToBuffers(string Model_Path, int max_tree_depth, int material_id)
        {
            GL.UseProgram(compute_shader);

            LoadObj.Load(new StreamReader(Model_Path));
            Triangle[] triangles = LoadObj.triangles.ToArray();

            try
            {
                BuildKDTree.LoadFromJson(new StreamReader(modelPath + ".tree"));
                Console.WriteLine("cached tree found");
            }
            catch
            {
                Console.WriteLine("building tree...");
                BuildKDTree.Build(triangles, max_tree_depth);
                BuildKDTree.CacheIntoJson(new StreamWriter(modelPath + ".tree"));
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

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "node_count"), BuildKDTree.preparedTreeData.nodes.Count);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 2, nodes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * BuildKDTree.preparedTreeData.nodes.Count, BuildKDTree.preparedTreeData.nodes.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 3, leaves);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * BuildKDTree.preparedTreeData.leaves.Count, BuildKDTree.preparedTreeData.leaves.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 4, triangle_indexes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * BuildKDTree.preparedTreeData.triangle_indexes_tree.Count, BuildKDTree.preparedTreeData.triangle_indexes_tree.ToArray(), BufferUsageHint.StaticDraw);

            List<Vector4> aabb_verts = new List<Vector4>();
            foreach (var aabb in BuildKDTree.preparedTreeData.aabbs)
            {
                aabb_verts.Add(new Vector4(aabb.min_x, aabb.min_y, aabb.min_z, 1));
                aabb_verts.Add(new Vector4(aabb.max_x, aabb.max_y, aabb.max_z, 1));
            }
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 5, aabbs);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(float) * 8 * BuildKDTree.preparedTreeData.aabbs.Count,
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