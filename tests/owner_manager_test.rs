use std::{cell::RefCell, rc::Rc, str::FromStr};

use anchor_lang::prelude::{AccountInfo, Pubkey};
use cartesi_solana::owner_manager;

use once_cell::sync::Lazy;

fn entry(accounts: &[AccountInfo]) {
    println!(" owner == {:?}", accounts[0].owner);
}

fn entry2(accounts: &[&Box<AccountInfo>]) {
    println!(" 2 owner == {:?}", accounts[0].owner);
}

#[test]
fn it_should_change_the_account_owner() {
    let owner: Pubkey = Pubkey::default();
    let key: &Pubkey = &Pubkey::default();
    let mut lamports: u64 = 1000;
    let mut info_data: Vec<u8> = Vec::new();
    let account_info: AccountInfo = AccountInfo {
        key,
        is_signer: true,
        is_writable: true,
        lamports: Rc::new(RefCell::new(&mut lamports)),
        data: Rc::new(RefCell::new(&mut info_data)),
        owner: &owner,
        executable: false,
        rent_epoch: 0,
    };
    let mut accounts = Vec::new();
    accounts.push(account_info);

    let p: *mut &Pubkey = std::ptr::addr_of_mut!(accounts[0].owner);
    println!("\n address {:?}", p);
    owner_manager::add_ptr(p as *mut Pubkey, *key);

    let new_owner: Pubkey = Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap();
    owner_manager::change_owner(key.clone(), new_owner);
    assert_eq!(
        accounts[0].owner.to_string(),
        "EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv"
    );
}

#[test]
fn i_should_learn_about_box() {
    let owner: Pubkey = Pubkey::default();
    let key = &Pubkey::default();
    let mut lamports: u64 = 1000;
    let mut info_data: Vec<u8> = Vec::new();
    let account_info: AccountInfo = AccountInfo {
        key,
        is_signer: true,
        is_writable: true,
        lamports: Rc::new(RefCell::new(&mut lamports)),
        data: Rc::new(RefCell::new(&mut info_data)),
        owner: &owner,
        executable: false,
        rent_epoch: 0,
    };
    let mut new_acc = account_info.clone();
    let new_owner = Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap();
    new_acc.owner = &new_owner;
    let mut b = Box::new(account_info);
    println!("b = {:?}", b);

    let accounts: &[AccountInfo] = &[*b];

    *b = new_acc;
    entry(&accounts);

    println!("owner = {:?}", b.owner);

    let accounts2: &[&Box<AccountInfo>] = &[&b];
    entry2(accounts2);
}

#[test]
fn it_should_change_the_owner_pointer_1() {
    let my_num: i32 = 10;
    let _my_num_ptr: *const i32 = &my_num;
    let mut my_speed: i32 = 88;
    {
        let my_speed_ptr: *mut i32 = &mut my_speed;
        let speed_ptr = unsafe { my_speed_ptr.as_mut().unwrap() };
        *speed_ptr = 55;
    }
    println!("\n {:?}", my_speed);
}

#[test]
fn it_should_change_the_owner_pointer_2() {
    let owner: Pubkey = Pubkey::default();
    let key = &Pubkey::default();
    let mut lamports: u64 = 1000;
    let mut info_data: Vec<u8> = Vec::new();
    let account_info: AccountInfo = AccountInfo {
        key,
        is_signer: true,
        is_writable: true,
        lamports: Rc::new(RefCell::new(&mut lamports)),
        data: Rc::new(RefCell::new(&mut info_data)),
        owner: &owner,
        executable: false,
        rent_epoch: 0,
    };
    let new_owner = Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap();
    let mut account_info2 = account_info.clone();
    account_info2.owner = &new_owner;

    let mut accounts = vec![account_info];
    let pointers = accounts.as_mut_ptr();
    unsafe {
        *pointers = account_info2;
    }
    assert_eq!(
        accounts[0].owner.to_string(),
        "EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv"
    );
}

#[test]
fn it_should_change_the_key() {
    // its like a pointer
    static mut IDS: Lazy<Vec<Pubkey>> = Lazy::new(|| {
        vec![Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap()]
    });
    let owner: Pubkey = Pubkey::default();
    let mut lamports: u64 = 1000;
    let mut info_data: Vec<u8> = Vec::new();
    let account_info: AccountInfo = unsafe {
        AccountInfo {
            key: &IDS[0],
            is_signer: true,
            is_writable: true,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            data: Rc::new(RefCell::new(&mut info_data)),
            owner: &owner,
            executable: false,
            rent_epoch: 0,
        }
    };
    assert_eq!(
        account_info.key.to_string(),
        "EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv"
    );
    unsafe {
        IDS[0] = Pubkey::from_str("2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv").unwrap();
    }

    assert_eq!(
        account_info.key.to_string(),
        "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv"
    );
}

#[test]
fn it_should_set_data_size() {
    let owner: Pubkey = Pubkey::default();
    let key = &Pubkey::default();
    let mut lamports: u64 = 1000;
    let mut info_data: Vec<u8> = Vec::new();
    let account_info: AccountInfo = AccountInfo {
        key,
        is_signer: true,
        is_writable: true,
        lamports: Rc::new(RefCell::new(&mut lamports)),
        data: Rc::new(RefCell::new(&mut info_data)),
        owner: &owner,
        executable: false,
        rent_epoch: 0,
    };
    owner_manager::set_data_size(&account_info, 10);
    assert_eq!(account_info.data.borrow().len(), 10);
}