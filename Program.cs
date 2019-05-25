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

            game.Run();
        }

        static int window_width = 640;
        static int window_height = 640;
        static int image_width = 640;
        static int image_height = 640;
        static int workgroup_size = 16;//max 32

        public Game() : base(window_width, window_height, new GraphicsMode(new ColorFormat(8, 8, 8, 0), 24, 8, 1/*msaa*/) , "PathTracing")
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
        Random rand = new Random();

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

            LoadPrimitivesToShader();

            texture = GL.GenTexture();
            GL.BindTexture(TextureTarget.Texture2D, texture);
            GL.TexStorage2D(TextureTarget2d.Texture2D, 1, SizedInternalFormat.Rgba8, image_width, image_height);
            GL.BindImageTexture(0, texture, 0, false, 0, TextureAccess.ReadWrite, SizedInternalFormat.Rgba8);
            GL.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureMagFilter, (int)All.Nearest);
            GL.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureMinFilter, (int)All.Nearest);

            GL.Uniform2(GL.GetUniformLocation(compute_shader, "resolution"), new Vector2(image_width, image_height));
            Matrix3 rotation_matrix = Matrix3.Identity;
            GL.UniformMatrix3(GL.GetUniformLocation(compute_shader, "rotation_mat"), false, ref rotation_matrix);           
        }

        public struct Triangle
        {
            public Vector3[] vertices;
        }

        void LoadPrimitivesToShader()
        {
            GL.UseProgram(compute_shader);

            //*****************************triangles*****************
            LoadObj.Load(new StreamReader("dragon.obj"));
            Triangle[] triangles = LoadObj.triangles.ToArray();

            Console.WriteLine("building tree...");

            BuildKDTree.Build(triangles, 10);

            Console.WriteLine("tree built");

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "triangles_amount"), triangles.Length);
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "spheres_amount"), 0);
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "planes_amount"), 6);

            
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
            { materials[i] = 3; }

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 1, triangle_materials);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(int) * materials.Length, materials, BufferUsageHint.StaticDraw);
            //*************triangle_tree*****************************
            int nodes = GL.GenBuffer(),
                leaves = GL.GenBuffer(),
                triangle_indexes = GL.GenBuffer(),
                aabbs = GL.GenBuffer();

            GL.Uniform1(GL.GetUniformLocation(compute_shader, "node_count"), BuildKDTree.Prepared_nodes.Count);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 2, nodes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, 
                sizeof(int) * 4 * BuildKDTree.Prepared_nodes.Count, BuildKDTree.Prepared_nodes.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 3, leaves);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * 4 * BuildKDTree.Prepared_leaves.Count, BuildKDTree.Prepared_leaves.ToArray(), BufferUsageHint.StaticDraw);

            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 4, triangle_indexes);
            GL.BufferData(BufferTarget.ShaderStorageBuffer,
                sizeof(int) * BuildKDTree.triangle_indexes_tree.Count, BuildKDTree.triangle_indexes_tree.ToArray(), BufferUsageHint.StaticDraw);

            List<Vector4> aabb_verts = new List<Vector4>();
            for(int i = 0; i < BuildKDTree.aabbs.Count; i++)
            {
                aabb_verts.Add(new Vector4(BuildKDTree.aabbs[i].min, 1));
                aabb_verts.Add(new Vector4(BuildKDTree.aabbs[i].max, 1));
            }
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 5, aabbs);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, sizeof(float) * 8 * BuildKDTree.aabbs.Count,
                aabb_verts.ToArray(), BufferUsageHint.StaticDraw);

            //********************************************************
        }

        int iterations = 0; 

        protected override void OnRenderFrame(FrameEventArgs E)
        {
            base.OnRenderFrame(E);

            GL.ClearColor(Color.Black);
            GL.Clear(ClearBufferMask.ColorBufferBit);

            GL.UseProgram(compute_shader);
            for (int i = 0; i < 1; i++)
            {
                GL.Uniform1(GL.GetUniformLocation(compute_shader, "rand_seed"), (rand.Next(100000)) / 100000f);
                iterations++;
                Console.WriteLine(iterations);
                GL.Uniform1(GL.GetUniformLocation(compute_shader, "iteration"), iterations);
                GL.DispatchCompute(image_width / workgroup_size, image_height / workgroup_size, 1);
                GL.MemoryBarrier(MemoryBarrierFlags.ShaderStorageBarrierBit);
            }

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
            {
                //save render to bitmap
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

                gamma_corrected_bitmap.Save("7.bmp", ImageFormat.Bmp);
                Console.WriteLine("saved");

                gamma_corrected_bitmap.Dispose();
                bmp.Dispose();
            }
        }
    }
}