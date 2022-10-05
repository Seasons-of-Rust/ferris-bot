fn main() {
    let mut a = 5;
    
    add_one(&mut a);

    println!("{}", a);
}

fn add_one(num: &mut i32) {
    *num += 1;
}
