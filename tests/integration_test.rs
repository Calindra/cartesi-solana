use anchor_lang::prelude::{AccountInfo, ProgramError, Pubkey};
use borsh::BorshSerialize;
use cartesi_solana::account_manager::{self, create_account_info, serialize_with_padding};
use solana_sdk::account::{ReadableAccount, WritableAccount};
use solana_sdk::{account::Account as Acc, account::AccountSharedData, account_info::Account};
use std::cell::RefMut;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

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
    //assert_eq!(account_info.data_len(), 64);
}

#[test]
fn it_should_serialize_with_shared_data() {
    let lamports = 1;
    let space = 42;
    let owner = Pubkey::default();
    let key = Pubkey::default();
    let asd = AccountSharedData::new(lamports, space, &owner);
    let mut account = Acc::from(asd);
    let (lamports, data, owner, executable, rent_epoch) = account.get();
    let account_info = AccountInfo::new(
        &key, false, true, lamports, data, owner, executable, rent_epoch,
    );

    let borsh_structure = BorshStructure {
        key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
    };
    let account_info2 = account_info.to_owned();
    let inner_data_vec = borsh_structure.try_to_vec().unwrap();
    assert_eq!(inner_data_vec.len(), 32);
    serialize_with_padding(&account_info2, &borsh_structure);

    assert_eq!(account_info.data_len(), 42);
    assert_eq!(account_info2.data_len(), 42);
    assert_eq!((*account_info.data.borrow())[0..32], inner_data_vec);
}

#[test]
fn it_should_serialize_with_shared_data_2() {
    let lamports = 1;
    let space = 42;
    let owner = Pubkey::default();
    let key = Pubkey::default();
    let asd = AccountSharedData::new(lamports, space, &owner);
    let mut account = Acc::from(asd);
    let (lamports, data, owner, executable, rent_epoch) = account.get();
    let account_info = AccountInfo::new(
        &key, false, true, lamports, data, owner, executable, rent_epoch,
    );

    let borsh_structure = BorshStructure {
        key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
    };

    account_manager::set_data_size(&account_info, 44);
    account_manager::set_data_size(&account_info, 42);

    let account_info2 = account_info.to_owned();
    let inner_data_vec = borsh_structure.try_to_vec().unwrap();
    assert_eq!(inner_data_vec.len(), 32);
    serialize_with_padding(&account_info2, &borsh_structure);

    assert_eq!(account_info.data_len(), 42);
    assert_eq!(account_info2.data_len(), 42);
    assert_eq!((*account_info.data.borrow())[0..32], inner_data_vec);
}

#[test]
fn it_should_serialize_with_shared_data_3() {
    let lamports = 1;
    let space = 32;
    let owner = Pubkey::default();
    let key = Pubkey::default();
    let mut asd = AccountSharedData::new(lamports, space, &owner);
    //let mut account = Acc::from(asd);
    //let (lamports, data, owner, executable, rent_epoch) = account.get();
    let executable = false;
    let rent_epoch = 1;
    let binding = asd.data_mut();
    // let account_info = AccountInfo::new(
    //     &key, false, true, &mut asd.lamports(), &mut binding, asd.owner(), executable, rent_epoch,
    // );

    // let borsh_structure = BorshStructure {
    //     key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
    // };

    // let account_info2 = account_info.to_owned();
    // let inner_data_vec = borsh_structure.try_to_vec().unwrap();
    // assert_eq!(inner_data_vec.len(), 32);
    // let new_data = vec![0u8; 32];
    // asd.set_data(new_data);
    // borsh_structure
    //     .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
    //     .unwrap();
    // account.data_mut().push(1);
    // assert_eq!(account.data[..32], inner_data_vec);
    //assert_eq!(account.data, inner_data_vec);

    // assert_eq!(account_info.data_len(), 42);
    // assert_eq!(account_info2.data_len(), 42);
    // assert_eq!((*account_info.data.borrow())[0..32], inner_data_vec);
}

#[test]
fn it_should_serialize_with_borsh() {
    let mut simple_array = [0u8; 5];
    let mut ref_simple: &mut [u8] = &mut simple_array;

    let new_data = vec![1, 2, 3];
    ref_simple.write_all(&new_data.as_ref()).unwrap();
    let new_data = vec![4, 5];
    ref_simple.write_all(&new_data.as_ref()).unwrap();

    assert_eq!(simple_array.len(), 5);
    assert_eq!(simple_array, [1, 2, 3, 4, 5])
}
