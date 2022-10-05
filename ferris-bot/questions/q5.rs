fn main() {
    let v = vec![1, 2, 3];

    drop(&v);
    drop(&v);
}
