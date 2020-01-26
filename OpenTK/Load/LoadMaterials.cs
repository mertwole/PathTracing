using OpenTK;
using OpenTK.Graphics.OpenGL4;

namespace PathTracing.Load
{
    public static class LoadMaterials
    {
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

        public static void Load()
        {
            Material[] materials = new Material[]
            {
                new Material() {color = new Vector3(1.0f, 1.0f, 1.0f), emission = new Vector3(1, 1, 1), emmisive = 1, reflective = 0, refractive = 0, refraction = 1.0f},//0
                new Material() {color = new Vector3(1.0f, 1.0f, 1.0f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 0, refractive = 0, refraction = 1.0f},//1
                new Material() {color = new Vector3(1.0f, 0.2f, 0.2f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 0, refractive = 0, refraction = 1.0f},//2
                new Material() {color = new Vector3(0.2f, 0.2f, 1.0f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 0, refractive = 0, refraction = 1.0f},//3
                new Material() {color = new Vector3(0.2f, 0.2f, 1.0f), emission = new Vector3(0, 0, 0), emmisive = 0, reflective = 1, refractive = 0, refraction = 1.0f},//4
            };

            int materials_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 2, materials_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, materials.Length * Material.size, materials, BufferUsageHint.StaticDraw);
        }
    }
}
