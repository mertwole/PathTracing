mod aabb;
mod obj_loader;
mod tree;
mod triangle;
mod vec3;

use crate::tree::*;

fn main() {
    let mut tree = Tree::new();

    load_triangles_from_files(&mut tree);

    let depth = ask_depth();
    tree.build(depth);

    tree.save("tree.txt");
}

fn ask_depth() -> u32 {
    println!("depth?");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap()
}

fn load_triangles_from_files(tree: &mut Tree) {
    println!("how much files?");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let num_files: usize = input.trim().parse().unwrap();
    println!("enter file names :");
    for _i in 0..num_files {
        input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        tree.load_triangles(&input[0..input.len() - 2].to_string());
    }
}
