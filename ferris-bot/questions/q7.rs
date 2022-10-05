struct Box {
    width: u32,
    height: u32,
    depth: u32,
}

impl Box {
    fn new(&self) -> Box {
        Box {
            width: 0,
            height: 0,
            depth: 0,
        }
    }
}

fn main() {
    let b = Box::new();
}
