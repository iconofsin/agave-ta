// Harness mostly lifted fro send_and_confirm_transactions_in_parallel

use {
    solana_client::{
        nonblocking::tpu_client::TpuClient,
        rpc_config::RpcSendTransactionConfig,
        send_and_confirm_transactions_in_parallel::{
            send_and_confirm_transactions_in_parallel_blocking_v2, send_and_confirm_transactions_in_parallel_v2, SendAndConfirmConfigV2
        },
    },
    solana_rpc_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, message::Message, native_token::sol_to_lamports,
        pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    },
    solana_streamer::socket::SocketAddrSpace,
    solana_test_validator::TestValidator,
    std::{path::Path, sync::Arc},
};


const NUM_TRANSACTIONS: usize = 32;

fn index_to_sol(i: usize) -> f64 {
    if i % 2 == 0 {
        10.001
    } else {
        9.999
    }
}

fn create_messages(from: Pubkey, to: Pubkey) -> (Vec<Message>, f64) {
    let mut messages = vec![];
    let mut sum = 0.0;
    for i in 1..NUM_TRANSACTIONS {
        let amount_to_transfer = index_to_sol(i);
        let ix = system_instruction::transfer(&from, &to, sol_to_lamports(amount_to_transfer));
        let message = Message::new(&[ix], Some(&from));
        messages.push(message);
        sum += amount_to_transfer;
    }
    (messages, sum)
}

// q&d
fn wallets<P: AsRef<Path> + ToString>(alice: P, bob: P) -> (Keypair, Pubkey) {
    let alice_json = std::fs::File::open(alice).unwrap();
    let alice_data: serde_json::Value = serde_json::from_reader(alice_json).unwrap();
    let alice_key_bytes: Vec<u8> = serde_json::from_value(alice_data).unwrap();    
    let alice_keypair = solana_sdk::signature::Keypair::from_bytes(&alice_key_bytes).unwrap();

    let bob_json = std::fs::File::open(bob).unwrap();
    let bob_data: serde_json::Value = serde_json::from_reader(bob_json).unwrap();
    let bob_key_bytes: Vec<u8> = serde_json::from_value(bob_data).unwrap();
    let bob_keypair = solana_sdk::signature::Keypair::from_bytes(&bob_key_bytes).unwrap();
    let bob_pubkey = bob_keypair.pubkey();

    (alice_keypair, bob_pubkey)
}

#[test]
fn test_logged_transactions() {
    solana_logger::setup();

    let rpc_url = "http://127.0.0.1:8899".to_owned();
    let ws_url = "ws://127.0.0.1:8900".to_owned();

    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    dbg!("RPC client created");

    let (alice, bob_pubkey) = wallets("../alice.json", "../bob.json");
    let alice_pubkey = alice.pubkey();

    dbg!("Wallets loaded");

    assert_eq!(
        rpc_client.get_version().unwrap().solana_core,
        solana_version::semver!()
    );

    let original_alice_balance = rpc_client.get_balance(&alice.pubkey()).unwrap();

    dbg!(original_alice_balance);

    let (messages, sum) = create_messages(alice_pubkey, bob_pubkey);

    dbg!("Transfer {} SOL from Alice to Bob {}", sum, messages.len());

    let tpu_client_fut = TpuClient::new(
        "temp",
        rpc_client.get_inner_client().clone(),
        ws_url.as_str(),
        solana_client::tpu_client::TpuClientConfig::default(),
    );
    let tpu_client = rpc_client.runtime().block_on(tpu_client_fut).unwrap();

    // solana_client::send_and_confirm_transactions_in_parallel::send_and_confirm_transactions_in_parallel_v2(rpc_client, tpu_client, messages, signers, config)
    
    let txs_errors = send_and_confirm_transactions_in_parallel_blocking_v2(
        rpc_client.clone(),
        Some(tpu_client),
        &messages,
        &[&alice],
        SendAndConfirmConfigV2 {
            with_spinner: true,
            resign_txs_count: Some(5),
            rpc_send_transaction_config: RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
        },
    );

    assert!(txs_errors.is_ok());
    assert!(txs_errors.unwrap().iter().all(|x| x.is_none()));

    // assert_eq!(
    //     rpc_client
    //         .get_balance_with_commitment(&bob_pubkey, CommitmentConfig::processed())
    //         .unwrap()
    //         .value,
    //     sol_to_lamports(sum)
    // );
    // assert_eq!(
    //     rpc_client
    //         .get_balance_with_commitment(&alice_pubkey, CommitmentConfig::processed())
    //         .unwrap()
    //         .value,
    //     original_alice_balance - sol_to_lamports(sum)
    // );
}
