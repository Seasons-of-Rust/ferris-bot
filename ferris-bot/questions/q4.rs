enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn main() {
    match Coin {
        Coin::Penny => println!("Penny"),
    }
}
