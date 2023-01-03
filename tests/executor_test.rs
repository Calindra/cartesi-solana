use anchor_lang::prelude::Pubkey;
use cartesi_solana::{
    executor::{Executor, LineReader},
    transaction::{self, Signature},
};
use solana_sdk::{
    hash::Hash,
    instruction::CompiledInstruction,
    message::{Message, MessageHeader},
};

use std::{fmt::{Write, format}, io, str::FromStr};

#[test]
fn it_should_create_account_infos() {
    let payload = create_payload();
    let instruction_index = 0;
    let timestamp = 123;
    let stdin = MyLineReader {
        lines: vec![
            "Header: External CPI".to_string(),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            payload,
            instruction_index.to_string(),
            timestamp.to_string(),
        ],
        current_line: 0,
    };
    let program_id = Some(Pubkey::default());
    let shared_data = vec![];
    let mut executor = Executor {
        stdin,
        program_id,
        shared_data,
        accounts_sdk: vec![],
        accounts_mut: vec![],
        accounts: vec![],
    };
    let (program_id, accounts, data) = executor.get_processor_args();
    assert_eq!(program_id.to_string(), "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv".to_string());
}

struct MyLineReader {
    pub lines: Vec<String>,
    pub current_line: usize,
}
impl LineReader for MyLineReader {
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let line_with_ender = format!("{}\n", &self.lines[self.current_line]);
        buf.write_str(&line_with_ender).unwrap();
        self.current_line += 1;
        Ok(1)
    }
}

fn create_payload() -> String {
    let signature: Signature = bincode::deserialize(&[0; 64]).unwrap();
    let signatures = vec![signature];
    let header = MessageHeader {
        num_required_signatures: 1,
        num_readonly_signed_accounts: 0,
        num_readonly_unsigned_accounts: 5,
    };
    let account_keys = vec![
        "1111111111112RXi1yn6kTp7G8Td7o6z3Ciqw9v2",
        "6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY",
        "11111111111111111111111111111111",
        "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv",
        "4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV",
        "SysvarRent111111111111111111111111111111111",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    ]
    .into_iter()
    .map(|str_pk| Pubkey::from_str(str_pk).unwrap())
    .collect();

    let recent_blockhash = Hash::default();

    let instruction = CompiledInstruction {
        program_id_index: 3,
        accounts: [1, 0, 4, 6, 2, 5].to_vec(),
        data: vec![141, 132, 233, 130, 168, 183, 10, 119],
    };
    let instructions = vec![instruction];

    let message = Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    };
    let transaction = transaction::Transaction {
        signatures,
        message,
    };
    let transaction_bytes = bincode::serialize(&transaction).unwrap();
    let base64_encoded = base64::encode(transaction_bytes);
    println!("base64 = {}", base64_encoded);

    base64_encoded
}
