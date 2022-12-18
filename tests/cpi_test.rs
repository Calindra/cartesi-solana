use std::str::FromStr;

use anchor_lang::{prelude::Pubkey, solana_program::{system_instruction::transfer, instruction::Instruction}};
use cartesi_solana::cpi::check_signature;

#[test]
fn cpi_it_should_verify_the_signature_success() {
    let program_id = Pubkey::default();
    let alice_pubkey = Pubkey::default();
    let seeds = &[b"escrow".as_ref()];
    let (escrow_pubkey, bump) = Pubkey::find_program_address(seeds, &program_id);

    let pda_signature = &[
        b"escrow".as_ref(),
        &[bump]
    ];
    // Create the transfer instruction
    let instruction = transfer(&escrow_pubkey, &alice_pubkey, 1);

    let signer_program_id = Pubkey::default();
    check_signature(&signer_program_id, &instruction, pda_signature);
}

#[test]
#[should_panic]
fn cpi_it_should_verify_the_signature_fail() {
    let program_id = Pubkey::default();
    let alice_pubkey = Pubkey::default();
    let seeds = &[b"escrow".as_ref()];
    let (escrow_pubkey, bump) = Pubkey::find_program_address(seeds, &program_id);

    let pda_signature = &[
        b"escrow".as_ref(),
        &[bump]
    ];
    // Create the transfer instruction
    let instruction = transfer(&escrow_pubkey, &alice_pubkey, 1);

    let signer_program_id = Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap();
    check_signature(&signer_program_id, &instruction, pda_signature);
}

