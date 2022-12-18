/// Cross-Program Invocations
/// 
use anchor_lang::{solana_program::instruction::Instruction, prelude::Pubkey};

pub fn check_signature(
    signer_program_id: &Pubkey,
    instruction: &Instruction,
    pda_signature: &[&[u8]],
) {
    for acc in instruction.accounts.iter() {
        if acc.is_signer {
            let expected =
                Pubkey::create_program_address(pda_signature, signer_program_id).unwrap();
            assert_eq!(expected, acc.pubkey);
        }
    }
}
