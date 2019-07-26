using System.Collections.Generic;
using System.IO;
using OpenTK;
using static Path_Tracing.Game;

namespace Path_Tracing
{
    class LoadObj
    {
        List<Vector3> normals = new List<Vector3>();
        List<Vector3> vertices = new List<Vector3>();      

        public List<Triangle> triangles = new List<Triangle>();

        public void Load(StreamReader stream)//3Ds max format
        {
            normals = new List<Vector3>();
            vertices = new List<Vector3>();
            triangles = new List<Triangle>();

            while (true)
            {
                string curr_line = stream.ReadLine();

                if (curr_line == null)
                    break;

                if (curr_line.Length < 3)
                    continue;

                if(curr_line.Substring(0, 2) == "v ")
                {//vertex
                    var vert = ParseStringToEnumarator(curr_line);

                    Vector3 new_vert = new Vector3();

                    vert.MoveNext();
                    new_vert.X = vert.Current;
                    vert.MoveNext();
                    new_vert.Y = vert.Current;
                    vert.MoveNext();
                    new_vert.Z = vert.Current;

                    vertices.Add(new_vert);
                }
            }

            stream.DiscardBufferedData();
            stream.BaseStream.Seek(0, SeekOrigin.Begin);

            while (true)
            {
                string curr_line = stream.ReadLine();

                if (curr_line == null)
                    break;

                if (curr_line.Length < 2)
                    continue;

                if(curr_line.Substring(0, 2) == "f ")
                {//triangle
                    Triangle triangle = new Triangle();
                    int[] verts = new int[3];

                    var face = ParseStringToEnumaratorI(curr_line);

                    face.MoveNext();
                    verts[0] = face.Current;
                    face.MoveNext();
                    face.MoveNext();
                    verts[1] = face.Current;
                    face.MoveNext();
                    face.MoveNext();
                    verts[2] = face.Current;

                    triangle.vertices = new Vector3[]
                    {
                        vertices[verts[0] - 1],
                        vertices[verts[1] - 1],
                        vertices[verts[2] - 1]
                    };

                    triangles.Add(triangle);
                }
            }

            stream.Close();
        }

        static List<char> allowed_chars = new List<char>() { '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ',', '-' };

        static IEnumerator<float> ParseStringToEnumarator(string input)
        {
            input = input + " ";

            input.Replace('.', ',');

            List<char> curr_num = new List<char>();

            foreach (char ch in input)
            {
                if (allowed_chars.Contains(ch))
                    curr_num.Add(ch);
                else if (curr_num.Count != 0)
                {
                    float.TryParse(new string(curr_num.ToArray()), out float i);
                    yield return i;
                    curr_num = new List<char>();
                }
            }
        }

        static List<char> allowed_charsI = new List<char>() { '0', '1', '2', '3', '4', '5', '6', '7', '8', '9' };

        static IEnumerator<int> ParseStringToEnumaratorI(string input)
        {
            input = input + " ";

            List<char> curr_num = new List<char>();

            foreach (char ch in input)
            {
                if (allowed_charsI.Contains(ch))
                    curr_num.Add(ch);
                else if (curr_num.Count != 0)
                {
                    int.TryParse(new string(curr_num.ToArray()), out int i);
                    yield return i;
                    curr_num = new List<char>();
                }
            }
        }
    }
}
