using OpenTK;
using OpenTK.Graphics.OpenGL4;
using System.Collections.Generic;
using System.Xml;

namespace PathTracing.Load
{
    public class LoadPlanes
    {
        struct Plane
        {
            public Vector3 normal;
            public float pad_0;
            public Vector3 point;
            public int material_id;

            public static int size = 8 * sizeof(float);
        }

        List<Plane> planes = new List<Plane>();

        public void Load(string xml_path)
        {
            XmlDocument xml = new XmlDocument();
            xml.Load(xml_path);
            ParseXML(xml.DocumentElement);

            GL.Uniform1(GL.GetUniformLocation(Game.compute_shader, "planes_amount"), planes.Count);
            int planes_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 1, planes_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, planes.Count * Plane.size, planes.ToArray(), BufferUsageHint.StaticDraw);
        }

        void ParseXML(XmlElement xml)
        {
            XmlNodeList plane_nodes = xml.ChildNodes;
            foreach (XmlNode plane_node in plane_nodes)
            {
                var new_plane = new Plane();

                new_plane.normal = CommonParse.ParseVector3(plane_node, "normal");
                new_plane.point = CommonParse.ParseVector3(plane_node, "point");

                new_plane.material_id = CommonParse.ParseInt(plane_node, "material");

                planes.Add(new_plane);
            }
        }
    }
}
