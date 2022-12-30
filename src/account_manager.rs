use anchor_lang::prelude::AccountInfo;
use anchor_lang::prelude::Pubkey;
use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::ErrorKind::NotFound;
use std::rc::Rc;
use std::{fs, str::FromStr};
// use solana_sdk::account::AccountSharedData;
static mut ACCOUNT_INFO_DATA: Vec<Vec<u8>> = Vec::new();
static mut MEM_DATA: Vec<AccountMemData> = Vec::new();

struct AccountMemData {
    key: Pubkey,
    owner: Pubkey,
    data: Box<[u8]>,
    lamports: u64,
}

pub fn clear() {
    unsafe {
        MEM_DATA.clear();
        ACCOUNT_INFO_DATA.clear();
    }
}

pub fn serialize_with_padding<B: BorshSerialize>(account_info: &AccountInfo, borsh_structure: &B) {
    let mut serialized_data = vec![0u8;0];
    borsh_structure.serialize(&mut serialized_data).unwrap();
    let diff = account_info.data_len() - serialized_data.len();
    for _ in 0..diff {
        serialized_data.push(0);
    }
    set_data(account_info, serialized_data);
}

pub fn set_data(account_info: &AccountInfo, data: Vec<u8>) {
    println!(
        "set_data: key = {:?}; data.len = {}",
        account_info.key,
        data.len()
    );
    if account_info.data_len() == data.len() {
        println!("set_data: account_info's data keep the same memory space");
        let mut data_info = account_info.try_borrow_mut_data().unwrap();
        for (i, byte) in data.iter().enumerate() {
            data_info[i] = *byte;
        }
    } else {
        unsafe {
            println!("set_data: RefCell replacing the account_info data");
            let tot = ACCOUNT_INFO_DATA.len();
            ACCOUNT_INFO_DATA.push(data);
            account_info.data.replace(&mut ACCOUNT_INFO_DATA[tot]);
        }
    }
}

pub fn set_data_size(account_info: &AccountInfo, size: usize) {
    unsafe {
        println!(
            "set_data_size: key = {:?}; size = {}",
            account_info.key, size
        );
        if account_info.data_len() != size {
            let tot = ACCOUNT_INFO_DATA.len();
            let data = vec![0; size];
            ACCOUNT_INFO_DATA.push(data);
            account_info.data.replace(&mut ACCOUNT_INFO_DATA[tot]);
        } else {
            println!("set_data_size: skipped");
        }
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
        let tot_mem_data = MEM_DATA.len();
        MEM_DATA.push(AccountMemData {
            key: key.to_owned(),
            owner,
            lamports,
            data: data.as_slice().into(),
        });
        let mem_data = &mut MEM_DATA[tot_mem_data];
        AccountInfo {
            key: &mem_data.key,
            is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(&mut mem_data.lamports)),
            data: Rc::new(RefCell::new(&mut mem_data.data)),
            owner: &mem_data.owner,
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
            //println!("base path from env {}", path);
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

#[derive(Debug)]
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
        println!(
            "saved {:?}; data.len() = {}",
            pubkey,
            account_file_data.data.len()
        );
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
