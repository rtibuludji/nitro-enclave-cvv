mod cvv;

use cvv::Cvv;

fn main() {
    println!("=== Example 1: DES key_a + DES key_b ===");
    let cvv = Cvv::new(
        "0123456789ABCDEF",
        "FEDCBA9876543210"
    ).expect("Failed to create CVV");


    let result1 = cvv.calculate(
        "4123456789012345",
        "8701",
        "101"
    ).expect("Failed to calculate CVV");

    let result2 = cvv.calculate(
        "4999988887777000",
        "9105",
        "111"
    ).expect("Failed to calculate CVV");

    let result3 = cvv.calculate(
        "4666655554444111",
        "9206",
        "120"
    ).expect("Failed to calculate CVV");

    println!("CVV = {}", result1);
    println!("CVV = {}", result2);
    println!("CVV = {}", result3);
    

    println!("Verify CVV = {:?}", cvv.verify("4666655554444111", "9206", "120", "664").expect("verify failed"));
    println!("Verify CVV = {:?}", cvv.verify("4666655554444111", "9206", "120", "665").expect("verify failed"));
    
}
