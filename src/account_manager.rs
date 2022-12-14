use anchor_lang::prelude::AccountInfo;
use anchor_lang::prelude::Pubkey;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::ErrorKind::NotFound;
use std::rc::Rc;
use std::{fs, str::FromStr};
static mut OWNERS: Lazy<Vec<Pubkey>> = Lazy::new(|| vec![]);
static mut ACCOUNT_INFO_DATA: Lazy<Vec<Vec<u8>>> = Lazy::new(|| vec![]);
static mut KEYS: Lazy<Vec<Pubkey>> = Lazy::new(|| vec![]);
static mut LAMPORTS: Lazy<Vec<u64>> = Lazy::new(|| vec![]);

pub fn clear() {
    unsafe {
        OWNERS.clear();
        KEYS.clear();
        ACCOUNT_INFO_DATA.clear();
        LAMPORTS.clear();
    }
}

pub fn set_data_size(account_info: &AccountInfo, size: usize) {
    unsafe {
        let tot = ACCOUNT_INFO_DATA.len();
        let data = vec![0; size];
        ACCOUNT_INFO_DATA.push(data);
        account_info.data.replace(&mut ACCOUNT_INFO_DATA[tot]);
    }
}

pub fn create_account_info<'a>(
    key: &Pubkey,
    is_signer: bool,
    is_writable: bool,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
    executable: bool,
) -> AccountInfo<'a> {
    unsafe {
        let tot_owners = OWNERS.len();
        OWNERS.push(owner);
        let tot_keys = KEYS.len();
        KEYS.push(key.to_owned());
        let tot_data = ACCOUNT_INFO_DATA.len();
        ACCOUNT_INFO_DATA.push(data);
        let tot_lamports = LAMPORTS.len();
        LAMPORTS.push(lamports);
        println!("create_account_info tot_keys = {} key = {:?} vs {:?}", tot_keys, &KEYS[tot_keys], &key);
        println!("First KEY = {:?}", &KEYS[0]);
        AccountInfo {
            key: &KEYS[tot_keys],
            is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(&mut LAMPORTS[tot_lamports])),
            data: Rc::new(RefCell::new(&mut ACCOUNT_INFO_DATA[tot_data])),
            owner: &OWNERS[tot_owners],
            executable,
            rent_epoch: 1,
        }
    }
}

pub fn create_account_manager() -> AccountManager {
    let mut account_manager = AccountManager::new().unwrap();
    let result = std::env::var("SOLANA_DATA_PATH");
    match result {
        Ok(path) => {
            println!("base path from env {}", path);
            account_manager.set_base_path(path);
            return account_manager;
        }
        Err(_) => {
            println!("default base path");
            account_manager.set_base_path("./".to_owned());
            return account_manager;
        }
    };
}

pub struct AccountManager {
    base_path: String,
}

impl AccountManager {
    pub fn new() -> std::result::Result<AccountManager, Box<dyn std::error::Error>> {
        Ok(Self {
            base_path: "tests/fixtures".to_string(),
        })
    }

    pub fn find_program_accounts(
        &self,
        pubkey: &Pubkey,
    ) -> std::result::Result<Vec<(Pubkey, AccountFileData)>, Box<dyn std::error::Error>> {
        let paths = fs::read_dir(&self.base_path)?;
        let mut result: Vec<(Pubkey, AccountFileData)> = vec![];
        for path in paths {
            let file_path = path?.path();
            let account_info = self.read_account_file(file_path.to_str().unwrap().to_string())?;
            if account_info.owner == *pubkey {
                let key = file_path.file_name().unwrap().to_str().unwrap();
                let pk = Pubkey::from_str(&key[..(key.len() - 5)]).unwrap();
                println!("program {:?} owns {:?}", &pubkey, &pk);
                result.push((pk, account_info));
            }
        }
        Ok(result)
    }

    pub fn set_base_path(&mut self, base_path: String) {
        self.base_path = base_path;
    }

    pub fn write_account(
        &self,
        pubkey: &Pubkey,
        account_file_data: &AccountFileData,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("{}/{}.json", &self.base_path, pubkey.to_string());
        let contents = serde_json::to_string(account_file_data)?;
        fs::write(file_path, contents)?;
        Ok(())
    }

    pub fn read_account(
        &self,
        pubkey: &Pubkey,
    ) -> std::result::Result<AccountFileData, Box<dyn std::error::Error>> {
        let file_path = format!("{}/{}.json", &self.base_path, pubkey.to_string());
        self.read_account_file(file_path)
    }

    fn read_account_file(
        &self,
        file_path: String,
    ) -> std::result::Result<AccountFileData, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(file_path)?;
        let account = serde_json::from_str::<AccountFileData>(&contents)?;
        Ok(account)
    }

    pub fn delete_account(
        &self,
        pubkey: &Pubkey,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("{}/{}.json", &self.base_path, pubkey.to_string());
        let delete_result = fs::remove_file(file_path);
        match delete_result {
            Ok(_) => {
                return Ok(());
            }
            Err(error) => {
                if error.kind() == NotFound {
                    return Ok(());
                } else {
                    return Err(Box::new(error));
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountFileData {
    /**
     * program owner
     */
    pub owner: Pubkey,
    pub data: Vec<u8>,
    pub lamports: u64,
}
