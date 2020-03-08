using OpenTK;
using OpenTK.Graphics.OpenGL4;
using System.Collections.Generic;
using System.Xml;

namespace PathTracing.Load
{
    public class LoadSpheres
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

        List<Sphere> spheres = new List<Sphere>();

        public void Load(string xml_path)
        {
            XmlDocument xml = new XmlDocument();
            xml.Load(xml_path);
            ParseXML(xml.DocumentElement);

            GL.Uniform1(GL.GetUniformLocation(Game.compute_shader, "spheres_amount"), spheres.Count);
            int spheres_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 0, spheres_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, spheres.Count * Sphere.size, spheres.ToArray(), BufferUsageHint.StaticDraw);
        }

        void ParseXML(XmlElement xml)
        {
            XmlNodeList sphere_nodes = xml.ChildNodes;
            foreach (XmlNode sphere_node in sphere_nodes)
            {
                var new_sphere = new Sphere();

                new_sphere.center = CommonParse.ParseVector3(sphere_node, "center");
                new_sphere.radius = CommonParse.ParseFloat(sphere_node, "radius");

                new_sphere.material = CommonParse.ParseInt(sphere_node, "material");

                spheres.Add(new_sphere);
            }
        }
    }
}
