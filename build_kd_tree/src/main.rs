extern crate kd_tree;

use kd_tree::tree::*;

fn main() {
    let mut tree = Tree::new();

    let path = ask_path();
    tree.load_triangles(&path);

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

fn ask_path() -> String {
    println!("enter file name :");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input[0..input.len() - 2].to_string()
}
