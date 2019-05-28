using System.Collections.Generic;
using System.IO;
using OpenTK;
using static Path_Tracing.Game;

namespace Path_Tracing
{
    class LoadObj
    {
        static List<Vector3> normals = new List<Vector3>();
        static List<Vector3> vertices = new List<Vector3>();      

        public static List<Triangle> triangles = new List<Triangle>();

        public static void Load(StreamReader stream)//3Ds max format
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

                if(curr_line.Substring(0, 3) == "v  ")
                {//vertex
                    curr_line = curr_line.Replace('.', ',');
                    //line is "v {x} {y} {z}"
                    curr_line = curr_line.Substring(3);
                    //line is "{x} {y} {z}"
                    int spacepos = curr_line.IndexOf(' ');
                    float.TryParse(curr_line.Substring(0, spacepos), out float x);
                    curr_line = curr_line.Substring(spacepos + 1, curr_line.Length - spacepos - 1);
                    //line is "{y} {z}"
                    spacepos = curr_line.IndexOf(' ');
                    float.TryParse(curr_line.Substring(0, spacepos), out float y);
                    curr_line = curr_line.Substring(spacepos + 1, curr_line.Length - spacepos - 1);
                    //line is "{z}"
                    float.TryParse(curr_line, out float z);

                    vertices.Add(new Vector3(x, y, z));
                }
                else if (curr_line.Substring(0, 3) == "vn ")
                {//normal
                    curr_line = curr_line.Replace('.', ',');
                    //line is "vn {x} {y} {z}"
                    curr_line = curr_line.Substring(3);
                    //line is "{x} {y} {z}"
                    int spacepos = curr_line.IndexOf(' ');
                    float.TryParse(curr_line.Substring(0, spacepos), out float x);
                    curr_line = curr_line.Substring(spacepos + 1, curr_line.Length - spacepos - 1);
                    //line is "{y} {z}"
                    spacepos = curr_line.IndexOf(' ');
                    float.TryParse(curr_line.Substring(0, spacepos), out float y);
                    curr_line = curr_line.Substring(spacepos + 1, curr_line.Length - spacepos - 1);
                    //line is "{z}"
                    float.TryParse(curr_line, out float z);

                    normals.Add(new Vector3(x, y, z));
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

                    //line is "f {vertex_num}//blah {vertex_num}//blah {vertex_num}//blah"
                    curr_line = curr_line.Substring(2);
                    //line is "{vertex_num}//blah {vertex_num}//blah {vertex_num}//blah"
                    int slash_pos = curr_line.IndexOf("//");
                    int space_pos = curr_line.IndexOf(' ');
                    int.TryParse(curr_line.Substring(0, slash_pos), out verts[0]);
                    curr_line = curr_line.Substring(space_pos + 1);
                    //line is "{vertex_num}//blah {vertex_num}//blah"
                    slash_pos = curr_line.IndexOf("//");
                    space_pos = curr_line.IndexOf(' ');
                    int.TryParse(curr_line.Substring(0, slash_pos), out verts[1]);
                    curr_line = curr_line.Substring(space_pos + 1);
                    //line is "{vertex_num}"
                    slash_pos = curr_line.IndexOf("//");
                    int.TryParse(curr_line.Substring(0, slash_pos), out verts[2]);

                    triangle.vertices = new Vector3[]
                    {
                        vertices[verts[0] - 1],
                        vertices[verts[1] - 1],
                        vertices[verts[2] - 1]
                    };

                    triangles.Add(triangle);
                }
            }
        }
    }
}
