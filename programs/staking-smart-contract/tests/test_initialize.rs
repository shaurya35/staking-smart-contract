use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn send_ix(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).unwrap();
}

#[test]
fn test_stake_and_unstake() {
    let program_id = staking_contract::id();
    let payer = Keypair::new();
    let (pda, _) = Pubkey::find_program_address(
        &[
            staking_contract::constants::STAKE_SEED,
            payer.pubkey().as_ref(),
        ],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/staking_contract.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &staking_contract::instruction::CreatePdaAccount {}.data(),
            staking_contract::accounts::CreatePdaAccount {
                payer: payer.pubkey(),
                pda_account: pda,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let stake_amount = 100_000_000u64;
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &staking_contract::instruction::Stake {
                amount: stake_amount,
            }
            .data(),
            staking_contract::accounts::Stake {
                user: payer.pubkey(),
                pda_account: pda,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let pda_account = svm.get_account(&pda).unwrap();
    let mut data: &[u8] = &pda_account.data;
    let stake_state = staking_contract::state::StakeAccount::try_deserialize(&mut data).unwrap();
    assert_eq!(stake_state.owner, payer.pubkey());
    assert_eq!(stake_state.staked_amount, stake_amount);

    let unstake_amount = 40_000_000u64;
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &staking_contract::instruction::Unstake {
                amount: unstake_amount,
            }
            .data(),
            staking_contract::accounts::Unstake {
                user: payer.pubkey(),
                pda_account: pda,
            }
            .to_account_metas(None),
        ),
    );

    let pda_account = svm.get_account(&pda).unwrap();
    let mut data: &[u8] = &pda_account.data;
    let stake_state = staking_contract::state::StakeAccount::try_deserialize(&mut data).unwrap();
    assert_eq!(
        stake_state.staked_amount,
        stake_amount - unstake_amount
    );
}
