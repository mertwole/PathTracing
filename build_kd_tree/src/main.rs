mod tree;
mod triangle;
mod aabb;
mod vec3;

use crate::tree::*;

fn main() {
    let mut tree = Tree::new();
    load_triangles_from_files(&mut tree);
    tree.build();
    tree.save();
}

fn load_triangles_from_files(tree : &mut Tree){
    println!("how much files?");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let num_files = input.parse::<u32>().unwrap();
    println!("enter file names :");
    for _i in 0..num_files{
        std::io::stdin().read_line(&mut input).unwrap();
        tree.load_triangles_obj(&input);
    }
}
