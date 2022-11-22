use std::env;
use std::str::FromStr;

use anchor_lang::prelude::Pubkey;
use cartesi_solana::account_manager::{AccountFileData, AccountManager};
use cartesi_solana::adapter::eth_address_to_pubkey;

fn create_account_manager(read_from_fixtures: bool) -> AccountManager {
    let dir = env::temp_dir();
    println!("Temporary directory: {}", dir.as_os_str().to_str().unwrap());
    let mut account_manager = AccountManager::new().unwrap();
    if !read_from_fixtures {
        account_manager.set_base_path(dir.as_os_str().to_str().unwrap().to_owned());
    }
    return account_manager;
}

#[test]
fn it_should_read_an_account_by_public_key() {
    let account_manager = create_account_manager(true);
    let pubkey = Pubkey::default();
    println!("key = {}", pubkey.to_string());
    let account_data = account_manager.read_account(&pubkey).unwrap();
    assert_eq!(account_data.lamports, 12345u64);
}

#[test]
fn it_should_write_an_account_info_by_public_key() {
    let pubkey = Pubkey::from_str("2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv").unwrap();
    println!("key = {}", pubkey.to_string());
    let mut data = Vec::new();
    data.push(1);
    let account_file_data = AccountFileData {
        owner: Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap(),
        data: data,
        lamports: 123,
    };
    let account_manager = create_account_manager(false);
    account_manager
        .write_account(&pubkey, &account_file_data)
        .unwrap();
    let account_data = account_manager.read_account(&pubkey).unwrap();
    assert_eq!(account_data.lamports, 123u64);
}

#[test]
fn it_should_delete_an_account_by_public_key() {
    let pubkey = Pubkey::from_str("2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv").unwrap();
    println!("key = {}", pubkey.to_string());
    let mut data = Vec::new();
    data.push(1);
    let account_file_data = AccountFileData {
        owner: Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap(),
        data: data,
        lamports: 123,
    };
    let account_manager = create_account_manager(false);
    account_manager
        .write_account(&pubkey, &account_file_data)
        .unwrap();
    account_manager.delete_account(&pubkey).unwrap();
}

#[test]
fn it_should_convert_eth_address_to_public_key() {
    // We implemented the same front behavior
    let bytes = hex::decode("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
    let pubkey = eth_address_to_pubkey(&bytes);
    assert_eq!(pubkey.to_string(), "1111111111112RXi1yn6kTp7G8Td7o6z3Ciqw9v2");
}
