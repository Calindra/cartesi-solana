use std::{time::{SystemTime, UNIX_EPOCH}, fs, str::FromStr};

use anchor_lang::prelude::Pubkey;
use cartesi_solana::{owner_manager, account_manager::{create_account_manager, AccountFileData}};

fn setup() {
    println!("\n\n***** setup *****\n");
    let dir = std::env::temp_dir();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let final_temp_dir = format!(
        "{}/{}",
        dir.as_os_str().to_str().unwrap(),
        since_the_epoch.subsec_nanos()
    );
    println!("{}", final_temp_dir);
    fs::create_dir(&final_temp_dir).unwrap();
    std::env::set_var("SOLANA_DATA_PATH", final_temp_dir);
    std::env::set_var("PORTAL_ADDRESS", "0xf8c694fd58360de278d5ff2276b7130bfdc0192a");
    unsafe {
        owner_manager::POINTERS.clear();
        owner_manager::OWNERS.clear();
    }
}

#[test]
fn it_should_write_read_and_delete_an_account() {
    setup();
    let account_manager = create_account_manager();
    let pubkey = Pubkey::default();
    let data = vec![];
    let account_file_data_to_write = AccountFileData {
        owner: pubkey,
        data,
        lamports: 1234u64,
    };
    account_manager.write_account(&pubkey, &account_file_data_to_write).unwrap();

    let account_file_data = account_manager.read_account(&pubkey).unwrap();
    assert_eq!(account_file_data.lamports, account_file_data_to_write.lamports);

    account_manager.delete_account(&pubkey).unwrap();

    let read_result = account_manager.read_account(&pubkey);
    match read_result {
        Ok(_) => panic!("Where is the file not found error??"),
        Err(e) => {
            if let Some(error) = e.downcast_ref::<std::io::Error>() {
                match error.kind() {
                    std::io::ErrorKind::NotFound => {
                        // ok
                    },
                    _ => {
                        panic!("Wrong error kind")
                    },
                }
            } else {
                panic!("Wrong error type")
            }
        },
    }
}

#[test]
fn it_should_list_all_program_accounts() {
    setup();
    let account_manager = create_account_manager();
    let pubkey = Pubkey::default();

    let data = vec![];
    let account_file_data_to_write = AccountFileData {

        // this is the program owner of the "file"
        owner: pubkey,

        data,
        lamports: 1234u64,
    };

    let file_key = Pubkey::from_str("EwiqbApgaLT2kQaohqZnSXT9HbkMQWDektXEjXGMJyJv").unwrap();
    account_manager.write_account(&file_key, &account_file_data_to_write).unwrap();

    let account_files = account_manager.find_program_accounts(&pubkey).unwrap();
    assert_eq!(account_files.len(), 1);
    assert_eq!(account_files[0].0, file_key);
}
