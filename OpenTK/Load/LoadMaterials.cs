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

                new_material.color = CommonParse.ParseVector3(material_node, "color");
                new_material.emission = CommonParse.ParseVector3(material_node, "emmision");

                new_material.emmisive = CommonParse.ParseFloat(material_node, "emmisive");
                new_material.reflective = CommonParse.ParseFloat(material_node, "reflective");
                new_material.refractive = CommonParse.ParseFloat(material_node, "refractive");
                new_material.refraction = CommonParse.ParseFloat(material_node, "refraction");

                materials.Add(new_material);
            }
        }
    }
}