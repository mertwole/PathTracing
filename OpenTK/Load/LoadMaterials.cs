using OpenTK;
using OpenTK.Graphics.OpenGL4;
using System.Collections.Generic;
using System.Xml;

namespace PathTracing.Load
{
    public class LoadMaterials
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

        List<Material> materials = new List<Material>();

        public void Load(string xml_path)
        {
            XmlDocument xml = new XmlDocument();
            xml.Load(xml_path);
            ParseXML(xml.DocumentElement);

            int materials_buffer = GL.GenBuffer();
            GL.BindBufferBase(BufferRangeTarget.ShaderStorageBuffer, 2, materials_buffer);
            GL.BufferData(BufferTarget.ShaderStorageBuffer, materials.Count * Material.size, materials.ToArray(), BufferUsageHint.StaticDraw);
        }

        void ParseXML(XmlElement xml)
        {
            XmlNodeList material_nodes = xml.ChildNodes;
            foreach(XmlNode material_node in material_nodes)
            {
                var new_material = new Material();

                new_material.color = ParseVector3(material_node, "color");
                new_material.emission = ParseVector3(material_node, "emmision");

                new_material.emmisive = ParseFloat(material_node, "emmisive");
                new_material.reflective = ParseFloat(material_node, "reflective");
                new_material.refractive = ParseFloat(material_node, "refractive");
                new_material.refraction = ParseFloat(material_node, "refraction");

                materials.Add(new_material);
            }
        }

        float ParseFloat(XmlNode node, string param_name)
        {
            float.TryParse(node.Attributes.GetNamedItem(param_name).InnerXml, out float output);
            return output;
        }

        Vector3 ParseVector3(XmlNode node, string param_name)
        {
            string attrib = node.Attributes.GetNamedItem(param_name).InnerXml;

            string[] coords = attrib.Split(' ');
            Vector3 vec = new Vector3();
            float.TryParse(coords[0], out vec.X);
            float.TryParse(coords[1], out vec.Y);
            float.TryParse(coords[2], out vec.Z);

            return vec;
        }
    }
}
