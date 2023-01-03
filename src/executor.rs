use std::io::Result;

use anchor_lang::prelude::{AccountInfo, Pubkey};
use solana_sdk::{account::Account as Acc, account::AccountSharedData, account_info::Account};

use crate::{adapter::check_header, transaction};

pub struct Executor<'a, LR: LineReader> {
    pub stdin: LR,
    pub program_id: Option<Pubkey>,
    pub shared_data: Vec<AccountSharedData>,
    pub accounts_sdk: Vec<Acc>,
    pub accounts_mut: Vec<&'a mut Acc>,
    pub accounts: Vec<AccountInfo<'a>>,
}

impl<'a, LR> Executor<'a, LR>
where
    LR: LineReader,
{
    pub fn get_processor_args(&'a mut self) -> (Pubkey, &Vec<AccountInfo<'a>>, Vec<u8>) {
        let header = self.read_line();
        println!("header: {}", header);
        match check_header(&header) {
            crate::adapter::SmartContractType::ExternalPI => self.handle_external_call(),
            crate::adapter::SmartContractType::CPI => todo!(),
        }
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

    fn handle_external_call(&'a mut self) -> (Pubkey, &Vec<AccountInfo<'a>>, Vec<u8>) {
        let msg_sender = self.read_line();
        let sender_bytes = self.sender_bytes(&msg_sender);
        let tx = self.read_transaction();
        let instruction_index = self.read_instruction_index();
        self.read_and_set_timestamp();
        let tx_instruction = &tx.message.instructions[instruction_index];
        let pidx: usize = (tx_instruction.program_id_index).into();
        let program_id: &Pubkey = &tx.message.account_keys[pidx];
        self.program_id = Some(program_id.to_owned());
        let program_id = self.program_id.unwrap();

        let instruction_data = vec![];

        static key: anchor_lang::prelude::Pubkey = Pubkey::new_from_array([0u8; 32]);
        let lamports = 1;
        let space = 0;
        let owner = Pubkey::default();
        let account_shared_data = AccountSharedData::new(lamports, space, &owner);
        self.shared_data.push(account_shared_data);
        for shared in self.shared_data.iter() {
            let account = Acc::from(shared.clone());
            self.accounts_sdk.push(account);
        }
        for acc in self.accounts_sdk.iter_mut() {
            self.accounts_mut.push(acc);
        }
        for acc in self.accounts_mut.iter_mut() {
            let (lamports, data, owner, executable, rent_epoch) = acc.get();
            let account_info = AccountInfo::new(
                &key, false, true, lamports, data, owner, executable, rent_epoch,
            );
            self.accounts.push(account_info);
        }
        (program_id, &self.accounts, instruction_data)
    }
}

pub trait LineReader {
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
}
