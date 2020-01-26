using OpenTK;
using OpenTK.Graphics.OpenGL4;

namespace PathTracing.Load
{
    public static class LoadPlanes
    {
        struct Plane
        {
            public Vector3 normal;
            public float pad_0;
            public Vector3 point;
            public int material_id;

            public static int size = 8 * sizeof(float);
        }

        public static void Load()
        {
            Plane[] planes = new Plane[]
            {
                new Plane() {normal = new Vector3(0, -1, 0).Normalized(), point = new Vector3(0, 3, 0), material_id = 0},//top
                
                new Plane() {normal = new Vector3(0, 1, 0).Normalized(), point = new Vector3(0, -3, 0), material_id = 1},//bottom
                
                new Plane() {normal = new Vector3(-1, 0, 0).Normalized(), point = new Vector3(3, 0, 0), material_id = 2},//right
                new Plane() {normal = new Vector3(1, 0, 0).Normalized(), point = new Vector3(-3, 0, 0), material_id = 3},//left

                new Plane() {normal = new Vector3(0, 0, 1).Normalized(), point = new Vector3(0, 0, -3), material_id = 1},//far
                new Plane() {normal = new Vector3(0, 0, -1).Normalized(), point = new Vector3(0, 0, 3), material_id = 1},//near
            };

            GL.Uniform1(GL.GetUniformLocation(Game.compute_shader, "planes_amount"), planes.Length);
            int planes_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 1, planes_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, planes.Length * Plane.size, planes, BufferUsageHint.StaticDraw);
        }
    }
}
