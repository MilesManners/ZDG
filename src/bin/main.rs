use zdg::node_map::NodeMap;

fn main() {
    let map = NodeMap::generate(10, 10, 5).unwrap();

    println!("{:?}", map);
}
