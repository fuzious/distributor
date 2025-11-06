use anchor_lang::system_program;

use crate::*;

pub fn process_clawback(args: &Args, clawback_args: &ClawbackArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    let merkle_tree_path = if let Some(version) = clawback_args.airdrop_version {
        let mut path = clawback_args.merkle_tree_path.clone();
        path.push(format!("tree_{}.json", version));
        path
    } else {
        // Fallback to reading the first merkle tree in the directory if no version is specified
        let mut paths: Vec<_> = fs::read_dir(&clawback_args.merkle_tree_path)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        paths.sort_by_key(|dir| dir.path());
        paths.first().unwrap().path()
    };

    let merkle_tree = AirdropMerkleTree::new_from_file(&merkle_tree_path)
        .expect("failed to load merkle tree from file");

    let (distributor, _bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

    loop {
        let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
        if distributor_state.clawed_back {
            println!("already clawback {}", merkle_tree.airdrop_version);
            break;
        }
        let clawback_ix = Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::Clawback {
                distributor,
                from: distributor_state.token_vault,
                token_program: spl_token::ID,
                to: distributor_state.clawback_receiver,
                claimant: keypair.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::Clawback {}.data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[clawback_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            client.get_latest_blockhash().unwrap(),
        );

        match client.send_transaction(&tx) {
            Ok(signature) => {
                println!(
                    "Successfully clawback airdrop version {} ! signature: {:#?}",
                    merkle_tree.airdrop_version,
                    signature
                );
                break;
            }
            Err(err) => {
                println!("airdrop version {} {}", merkle_tree.airdrop_version, err);
            }
        }
    }
}
