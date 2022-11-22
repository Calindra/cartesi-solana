use cartesi_solana::anchor_lang::prelude::Rent;


#[test]
fn it_should_calc_rent() {
    let rent = Rent::get().unwrap();
    let min = rent.minimum_balance(10);
    let is_ok = rent.is_exempt(min, 10);
    assert!(is_ok);
}
