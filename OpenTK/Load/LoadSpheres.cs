using OpenTK;
using OpenTK.Graphics.OpenGL4;

namespace PathTracing.Load
{
    public static class LoadSpheres
    {
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

        public static void Load()
        {
            Sphere[] spheres = new Sphere[]
            {
                new Sphere() {center = new Vector3(3, -3, 1), radius = 1, material = 0 },
                new Sphere() {center = new Vector3(0, -1, 0), radius = 1, material = 4 },
            };

            GL.Uniform1(GL.GetUniformLocation(Game.compute_shader, "spheres_amount"), spheres.Length);
            int spheres_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 0, spheres_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, spheres.Length * Sphere.size, spheres, BufferUsageHint.StaticDraw);
        }
    }
}
