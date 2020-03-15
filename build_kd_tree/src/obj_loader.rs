use crate::triangle::*;
use crate::vec3::*;

use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

pub struct OBJLoader {}

impl OBJLoader {
    pub fn load_obj(path: &String, out_triangles: &mut Vec<Triangle>){           
        OBJLoader::load_triangles(path, out_triangles);
    }

    fn load_triangles(path : &String, out_triangles: &mut Vec<Triangle>){
        let reader = BufReader::new(File::open(path.as_str()).unwrap()); 
        let verts = OBJLoader::load_verts(reader);
        let reader = BufReader::new(File::open(path.as_str()).unwrap()); 

        for line in reader.lines() {
            let line = line.unwrap();
            if line.len() < 2 { continue; }
            if line.chars().nth(0).unwrap() == 'f' && line.chars().nth(1).unwrap() == ' '{
                let ids : String = line.chars().into_iter().skip(2).collect();
                let mut ids_iter = ids.split_whitespace();

                fn get_first_num(line : &str) -> usize{
                    line.split("/").next().unwrap().parse::<usize>().unwrap()
                }

                let mut ids : [usize; 3] = [0, 0, 0];
                for i in 0..3{
                    ids[i] = get_first_num(ids_iter.next().unwrap());
                }
                out_triangles.push(Triangle {points : [verts[ids[0] - 1].clone(), verts[ids[0] - 1].clone(), verts[ids[0] - 1].clone()]});
            }
        }
    }

    fn load_verts(reader : std::io::BufReader<std::fs::File>) -> Vec<Vec3>{
        let mut verts : Vec<Vec3> = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();   
            if line.len() < 2 { continue; }       
            if line.chars().nth(0).unwrap() == 'v' && line.chars().nth(1).unwrap() == ' '{
                let coords : String = line.chars().into_iter().skip(2).collect();
                let mut coords_iter = coords.split_whitespace();
                let x = coords_iter.next().unwrap().parse::<f32>().unwrap();
                let y = coords_iter.next().unwrap().parse::<f32>().unwrap();
                let z = coords_iter.next().unwrap().parse::<f32>().unwrap();
                let mut vert = Vec3::new(x, y, z);
                match coords_iter.next(){
                    Some(w) => { vert = &vert / w.parse::<f32>().unwrap(); },
                    _=>{}
                }
                verts.push(vert);
            }
        }

        verts
    }
}
