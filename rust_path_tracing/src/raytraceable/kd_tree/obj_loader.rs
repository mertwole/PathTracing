use crate::raytraceable::triangle::Triangle;
use math::Vec3;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub struct OBJLoader {}

impl OBJLoader {
    pub fn load_obj(path: &str, out_triangles: &mut Vec<Triangle>) {
        OBJLoader::load_triangles(path, out_triangles);
    }

    fn load_triangles(path: &str, out_triangles: &mut Vec<Triangle>) {
        let reader = BufReader::new(File::open(path).unwrap());
        let verts = OBJLoader::load_verts(reader);

        let reader = BufReader::new(File::open(path).unwrap());
        let normals = OBJLoader::load_normals(reader);

        let reader = BufReader::new(File::open(path).unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            if line.len() < 2 {
                continue;
            }
            if line.starts_with('f') && line.chars().nth(1).unwrap() == ' ' {
                let face_descr: String = line.chars().into_iter().skip(2).collect();
                let mut face_descr_iter = face_descr.split_whitespace();

                let mut vert_ids: [usize; 3] = [0, 0, 0];
                let mut normal_ids: [usize; 3] = [0, 0, 0];
                for i in 0..3 {
                    let ids_line = face_descr_iter.next().unwrap();
                    let mut ids_iter = ids_line.split('/');
                    // Vert
                    vert_ids[i] = ids_iter.next().unwrap().parse::<usize>().unwrap();
                    // Tex
                    ids_iter.next().unwrap();
                    // Norm
                    normal_ids[i] = ids_iter.next().unwrap().parse::<usize>().unwrap();
                }
                let new_tr_points = [
                    verts[vert_ids[0] - 1],
                    verts[vert_ids[1] - 1],
                    verts[vert_ids[2] - 1],
                ];
                let new_tr_normal = normals[normal_ids[0] - 1];
                let new_tr = Triangle::new(new_tr_points, new_tr_normal, 0);
                out_triangles.push(new_tr);
            }
        }
    }

    fn load_verts(reader: std::io::BufReader<std::fs::File>) -> Vec<Vec3> {
        let mut verts: Vec<Vec3> = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();
            if line.len() < 2 {
                continue;
            }
            if line.starts_with('v') && line.chars().nth(1).unwrap() == ' ' {
                let coords: String = line.chars().into_iter().skip(2).collect();
                verts.push(OBJLoader::parse_vec3(&coords));
            }
        }

        verts
    }

    fn load_normals(reader: std::io::BufReader<std::fs::File>) -> Vec<Vec3> {
        let mut normals: Vec<Vec3> = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();
            if line.len() < 2 {
                continue;
            }
            if line.starts_with('v') && line.chars().nth(1).unwrap() == 'n' {
                let coords: String = line.chars().into_iter().skip(3).collect();
                normals.push(OBJLoader::parse_vec3(&coords).normalized());
            }
        }

        normals
    }

    fn parse_vec3(line: &str) -> Vec3 {
        let mut coords_iter = line.split_whitespace();
        let x = coords_iter.next().unwrap().parse::<f32>().unwrap();
        let y = coords_iter.next().unwrap().parse::<f32>().unwrap();
        let z = coords_iter.next().unwrap().parse::<f32>().unwrap();
        let mut point = Vec3::new(x, y, z);
        if let Some(w) = coords_iter.next() {
            point = point / w.parse::<f32>().unwrap();
        }
        point
    }
}
