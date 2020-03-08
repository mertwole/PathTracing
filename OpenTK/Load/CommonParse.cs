using OpenTK;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Xml;

namespace PathTracing.Load
{
    internal static class CommonParse
    {
        internal static float ParseFloat(XmlNode node, string param_name)
        {
            float.TryParse(node.Attributes.GetNamedItem(param_name).InnerXml, out float output);
            return output;
        }

        internal static int ParseInt(XmlNode node, string param_name)
        {
            int.TryParse(node.Attributes.GetNamedItem(param_name).InnerXml, out int output);
            return output;
        }

        internal static Vector3 ParseVector3(XmlNode node, string param_name)
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
