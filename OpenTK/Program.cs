using System;
using OpenTK;
using OpenTK.Input;
using OpenTK.Graphics;
using OpenTK.Graphics.OpenGL4;
using System.IO;
using System.Drawing;
using System.Drawing.Imaging;
using PixelFormat = OpenTK.Graphics.OpenGL4.PixelFormat;
using PathTracing.Load;

namespace PathTracing
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
            GL.Viewport(ClientRectangle.X, ClientRectangle.Y, ClientRectangle.Width, ClientRectangle.Height);
        }

        public static int compute_shader;
        int render_shader;
        int VAO, VBO;
        int texture;

        protected override void OnLoad(EventArgs E)
        {
            #region screen quad
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
            #endregion

            render_shader = CompileShaders.Compile(new StreamReader("frag_shader.glsl"), new StreamReader("vert_shader.glsl"));
            compute_shader = CompileShaders.CompileComputeShader(new StreamReader("comp_shader.glsl"));

            GL.UseProgram(compute_shader);

            #region texture
            texture = GL.GenTexture();
            GL.BindTexture(TextureTarget.Texture2D, texture);
            GL.TexStorage2D(TextureTarget2d.Texture2D, 1, SizedInternalFormat.Rgba8, image_width, image_height);
            GL.TextureParameter(texture, TextureParameterName.TextureMinFilter, (int)All.Linear);
            GL.TextureParameter(texture, TextureParameterName.TextureMagFilter, (int)All.Linear);
            GL.BindImageTexture(0, texture, 0, false, 0, TextureAccess.ReadWrite, SizedInternalFormat.Rgba8);
            #endregion

            #region camera
            GL.Uniform2(GL.GetUniformLocation(compute_shader, "resolution"), new Vector2(image_width, image_height));
            Matrix3 rotation_matrix = Matrix3.Identity;
            GL.UniformMatrix3(GL.GetUniformLocation(compute_shader, "rotation_mat"), false, ref rotation_matrix);
            GL.Uniform3(GL.GetUniformLocation(compute_shader, "view_point"), new Vector3(0, 0, 10));
            GL.Uniform1(GL.GetUniformLocation(compute_shader, "view_distance"), 7.01f);
            GL.Uniform2(GL.GetUniformLocation(compute_shader, "viewport"), new Vector2(5.99f, 5.99f));
            #endregion

            LoadPlanes.Load();
            LoadSpheres.Load();
            LoadMaterials.Load();
            LoadModel.Load("stanford-dragon", 18, 1);
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
            for(int i = 0; i < 10; i++)
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

            gamma_corrected_bitmap.Save("output.bmp", ImageFormat.Bmp);
            Console.WriteLine("saved");

            gamma_corrected_bitmap.Dispose();
            bmp.Dispose();
        }
    }
}