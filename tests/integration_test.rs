use std::str::FromStr;

use anchor_lang::prelude::{borsh, Pubkey};
use borsh::BorshSerialize;
use cartesi_solana::account_manager::create_account_info;

#[derive(BorshSerialize)]
struct BorshStructure {
    key: Pubkey,
}

#[test]
fn it_should_serialize_to_vec_u8() {
    let borsh_structure = BorshStructure {
        key: Pubkey::default(),
    };
    let mut writer: Vec<u8> = vec![0u8; 0];
    borsh_structure.serialize(&mut writer).unwrap();
    assert_eq!(writer.len(), 32);
}

#[test]
fn it_should_serialize_to_account_info() {
    let borsh_structure = BorshStructure {
        key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
    };
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let data = vec![0u8; 64];
    let account_info = create_account_info(&key, true, true, 1, data.to_owned(), owner, false);
    assert_eq!(account_info.data_len(), 64);

    borsh_structure
        .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
        .unwrap();
    println!("data = {:?}; data = {:?}", account_info.data, data);
    assert_eq!(account_info.data_len(), 64);
}
