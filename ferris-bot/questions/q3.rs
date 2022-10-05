fn main() {
    println!("{}", add_one(&mut 1));
}

fn add_one(num: &mut i32) {
    *num += 1;
}
