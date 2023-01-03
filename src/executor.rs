use std::{
    cell::RefCell,
    io::{self, Result},
    rc::Rc,
};

use anchor_lang::prelude::{AccountInfo, Pubkey};
use solana_sdk::instruction::CompiledInstruction;

use crate::{
    account_manager::{self, create_account_manager, AccountFileData},
    adapter::{check_header, check_signer_by_sender, load_account_info_data},
    owner_manager, transaction, cartesi_stub::CartesiStubs, anchor_lang::solana_program,
};

struct DataHolder {
    pubkey: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
}

pub struct Executor<'a, LR: LineReader> {
    pub stdin: LR,
    pub program_id: Option<Pubkey>,
    pub accounts: Vec<AccountInfo<'a>>,
    pub account_keys: Vec<Pubkey>,
}

impl<'a, LR> Executor<'a, LR>
where
    LR: LineReader,
{
    pub fn create_with_stdin(stdin: LR) -> Self {
        let program_id = Some(Pubkey::default());

        Self {
            stdin,
            program_id,
            accounts: vec![],
            account_keys: vec![],
        }
    }
    pub fn get_processor_args<F>(&'a mut self, f: F)
    where
        F: Fn(Pubkey, &Vec<AccountInfo>, Vec<u8>),
    {
        let header = self.read_line();
        println!("header: {}", header);
        match check_header(&header) {
            crate::adapter::SmartContractType::ExternalPI => self.handle_external_call(f),
            crate::adapter::SmartContractType::CPI => todo!(),
        };
    }

    fn read_line(&mut self) -> String {
        let mut current_line = String::new();
        self.stdin.read_line(&mut current_line).unwrap();
        current_line[..current_line.len() - 1].to_string()
    }

    fn read_instruction_index(&mut self) -> usize {
        let instruction_index = self.read_line();
        let instruction_index: usize = instruction_index
            .trim()
            .parse()
            .expect("Input is not an integer");
        instruction_index
    }

    fn read_and_set_timestamp(&mut self) {
        let timestamp = self.read_line();
        let timestamp: i64 = timestamp
            .trim()
            .parse()
            .expect("Timestamp is not an integer");
        unsafe {
            crate::anchor_lang::TIMESTAMP = timestamp;
        }
    }

    fn read_transaction(&mut self) -> transaction::Transaction {
        let payload = self.read_line();
        let decoded = base64::decode(payload).unwrap();
        let tx: transaction::Transaction = bincode::deserialize(&decoded).unwrap();
        tx
    }

    fn sender_bytes(&mut self, msg_sender: &String) -> Vec<u8> {
        let sender_bytes: Vec<u8> = hex::decode(&msg_sender[2..])
            .unwrap()
            .into_iter()
            .rev()
            .collect();
        sender_bytes
    }

    fn load_persisted_data(&self, account_keys: &Vec<Pubkey>) -> Vec<DataHolder> {
        let mut data_holder = vec![];
        for (i, pkey) in account_keys.iter().enumerate() {
            let (data, lamports, owner) = load_account_info_data(&pkey);
            println!(
                "loading account[{}] with key = {:?}; data.len() = {}; program_id = {:?}",
                i,
                &pkey,
                data.len(),
                self.program_id
            );
            data_holder.push(DataHolder {
                pubkey: pkey.to_owned(),
                lamports,
                data,
                owner,
            });
        }
        data_holder
    }

    fn get_ordered_account_keys(
        &mut self,
        tx: &transaction::Transaction,
        tx_instruction: &CompiledInstruction,
    ) -> Vec<Pubkey> {
        let mut ordered_accounts: Vec<Pubkey> = Vec::new();
        let tot = tx_instruction.accounts.len();
        for j in 0..tot {
            let index = tx_instruction.accounts[j];
            let i: usize = index.into();
            ordered_accounts.push(tx.message.account_keys[i].to_owned());
        }
        ordered_accounts
    }

    fn handle_external_call<F>(&'a mut self, closure_fn: F)
    where
        F: Fn(Pubkey, &Vec<AccountInfo>, Vec<u8>),
    {
        let msg_sender = self.read_line(); // the order of read commands is important!
        let sender_bytes = self.sender_bytes(&msg_sender);
        let tx = self.read_transaction();
        let instruction_index = self.read_instruction_index();
        self.read_and_set_timestamp();
        let tx_instruction = &tx.message.instructions[instruction_index];
        let pidx: usize = (tx_instruction.program_id_index).into();
        let program_id: &Pubkey = &tx.message.account_keys[pidx];
        solana_program::program_stubs::set_syscall_stubs(Box::new(CartesiStubs { program_id: program_id.clone() }));

        self.program_id = Some(program_id.to_owned());
        let program_id = self.program_id.unwrap();
        let ordered_accounts = self.get_ordered_account_keys(&tx, tx_instruction);

        let mut data_holder = self.load_persisted_data(&ordered_accounts);
        let mut accounts = vec![];
        for holder in data_holder.iter_mut() {
            let key = &holder.pubkey;
            let is_signer = check_signer_by_sender(key, &sender_bytes);
            let account_info = AccountInfo {
                key: &holder.pubkey,
                is_signer,
                is_writable: true,
                lamports: Rc::new(RefCell::new(&mut holder.lamports)),
                data: Rc::new(RefCell::new(&mut holder.data)),
                owner: &holder.owner,
                executable: false,
                rent_epoch: 1,
            };
            accounts.push(account_info);
        }

        // the addresses changes when you push to vec
        // so we need to get the pointers here, after
        let tot = accounts.len();
        for j in 0..tot {
            let p: *mut &Pubkey = std::ptr::addr_of_mut!(accounts[j].owner);
            owner_manager::add_ptr(p as *mut Pubkey, accounts[j].key.clone());
        }

        closure_fn(program_id, &accounts, tx_instruction.data.to_owned());
        let new_owners: Vec<Pubkey> = accounts
            .iter()
            .map(|account| account.owner.to_owned())
            .collect();
        persist_accounts(data_holder, new_owners);
    }
}

fn persist_accounts(data_holder: Vec<DataHolder>, new_owners: Vec<Pubkey>) {
    let account_manager = create_account_manager();
    for (i, holder) in data_holder.iter().enumerate() {
        let key = &holder.pubkey;
        let res = account_manager::get_resized(key);
        let mut final_data = holder.data.to_owned();
        if let Some(data) = res {
            final_data = data;
        }
        let account_file_data = AccountFileData {
            owner: new_owners[i].to_owned(),
            data: final_data.to_owned(),
            lamports: holder.lamports,
        };
        if account_file_data.lamports <= 0 {
            account_manager.delete_account(&key).unwrap();
            println!("! deleted = {:?}", key);
        } else {
            account_manager
                .write_account(&key, &account_file_data)
                .unwrap();
            println!("   saved = {:?};", key);
            println!("     owner = {:?}", account_file_data.owner.to_string());
        }
    }
}

pub trait LineReader {
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
}

pub struct DefaultStdin {}

impl LineReader for DefaultStdin {
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        return std::io::stdin().read_line(buf);
    }
}
