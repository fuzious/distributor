# Merkle Distributor Setup Guide

This guide provides a comprehensive walkthrough on how to set up and use the Solana Merkle Distributor program from scratch.

## Important
-   **VERIFY ON-CHAIN DATA**: After creating a distributor, always verify the on-chain account data to ensure the `admin`, `clawback_receiver`, and merkle `root` match your intended configuration. This is a critical step to prevent front-running attacks where a malicious actor could replace these values.
-   **CRITICAL**: Recheck the `admin` and `clawback_receiver` addresses in the smart contracts and CLI commands before deploying to mainnet to avoid loss of funds.

## Step 1: Prepare the Recipients CSV

The first step is to create a CSV file that lists the recipients and the amount of tokens they are eligible to claim. The CSV file must have a header row with the following columns: `pubkey`, `amount`, and `locked_amount`.

-   `pubkey`: The Solana public key of the recipient.
-   `amount`: The number of tokens to be released gradually. 50% is available at the start of the vesting period, and the rest unlocks linearly until the end.
-   `locked_amount`: The number of tokens to be released all at once after the vesting period ends (a "cliff" unlock).

### Example CSV for Immediate Full Unlock

To release all tokens at once after a specific time, set the `amount` column to `0` and put the token amounts in the `locked_amount` column.

**`recipients.csv`:**
```csv
pubkey,amount,locked_amount
H5g6EbMmxQBuPmjuGfkwmfNC5mPKj4jpneJpSy3BXxYW,0,2
7Gzxb9aeSYu7f91y583aa2iQ7XZKMg7M2a7JfAiypc6N,0,3
```

## Step 2: Create the Merkle Tree

Once the CSV file is ready, you can create a merkle tree from it. This is done using the `create-merkle-tree` command.

```bash
source ~/.bashrc && cargo run --bin cli -- create-merkle-tree \
    --csv-path recipients.csv \
    --merkle-tree-path merkle_trees \
    --max-nodes-per-tree 1000000 \
    --decimals 9 \
    --start-airdrop-version 6 \
    --amount 0
```

-   `--csv-path`: Path to your `recipients.csv` file.
-   `--merkle-tree-path`: Directory where the generated merkle tree JSON file will be saved.
-   `--max-nodes-per-tree`: The maximum number of recipients per merkle tree.
-   `--decimals`: The number of decimals for your token.
-   `--start-airdrop-version`: The version number for this airdrop.
-   `--amount`: A default amount, required by the CLI. Set to `0` if all amounts are in the `locked_amount` column.

## Step 3: Create the New Distributor

Next, create the on-chain distributor account. This requires setting the vesting period and other parameters.

First, get the current Unix timestamp:
```bash
date +%s
```

Then, use this timestamp to set the vesting period in the near future (e.g., 60 seconds from now).

```bash
source ~/.bashrc && cargo run --bin cli -- \
    --mint Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr \
    --rpc-url https://api.mainnet-beta.solana.com \
    --program-id 6YbCoGRzG1K9PouQKNrs3bMnHye77FsiWCTLYnU59TRU \
    --keypair-path ~/.config/solana/id.json \
    new-distributor \
    --merkle-tree-path merkle_trees \
    --start-vesting-ts <start_timestamp> \
    --end-vesting-ts <end_timestamp> \
    --clawback-start-ts <clawback_timestamp> \
    --clawback-receiver-owner 6dfrn1uceBjjUi6HMsJimgcC4qaGbPQQtsHC1Wtwhct1 \
    --enable-slot 0 \
    --closable \
    --airdrop-version 5
```

-   `--mint`: The token mint address.
-   `--rpc-url`: The Solana RPC endpoint.
-   `--program-id`: The on-chain program ID.
-   `--keypair-path`: Path to your admin keypair. This keypair will pay for the transaction and become the `admin` for this distributor.
-   `--merkle-tree-path`: Directory containing the merkle tree JSON file for the specified airdrop version.
-   `--start-vesting-ts`: The Unix timestamp when the vesting period begins.
-   `--end-vesting-ts`: The Unix timestamp when the vesting period ends. For an immediate unlock, set this to `start_vesting_ts + 1`.
-   `--clawback-start-ts`: The Unix timestamp when tokens can be clawed back. Must be at least one day after `end_vesting_ts`.
-   `--clawback-receiver-owner` (Optional): The public key of the wallet that will own the clawback token account. If not provided, this will default to the admin wallet specified by `--keypair-path`.
-   `--enable-slot`: The Solana slot number from which claiming is enabled. `0` means it's enabled immediately.
-   `--closable`: A flag to indicate that the distributor can be closed for emergency withdrawals.
-   `--airdrop-version`: The version of the airdrop, which must match the merkle tree.

> **⚠️ Security Warning**
> After running the `new-distributor` command, your transaction could be susceptible to front-running. It is crucial to immediately verify that the on-chain `MerkleDistributor` account was created with the correct parameters (especially `admin`, `clawback_receiver`, and `root`). If the transaction fails or the on-chain data is incorrect, do not proceed with funding the token vault.

## Step 4: Fund the Distributor's Token Vault

After creating the distributor, you need to fund its token vault.

1.  **Get the Distributor PDA**: The Program Derived Address (PDA) for the distributor is derived from the program ID, mint, and airdrop version. You can get this by re-running the `new-distributor` command, which will detect that the account exists and print the PDA.

2.  **Find the Token Vault ATA**: Once you have the distributor PDA, find its Associated Token Account (ATA) for your mint.

    ```bash
    source ~/.bashrc && spl-token accounts --owner <DISTRIBUTOR_PDA>
    ```

3.  **Transfer Tokens**: Transfer the total amount of tokens from your admin wallet to the distributor's token vault ATA.

    ```bash
    source ~/.bashrc && spl-token transfer <MINT_ADDRESS> <TOTAL_AMOUNT> <VAULT_ATA> --owner <ADMIN_KEYPAIR_PATH> --fund-recipient
    ```

## Step 5: Claim Tokens

Once the distributor is funded and the vesting period has passed, recipients can claim their tokens.

```bash
source ~/.bashrc && cargo run --bin cli -- \
    --mint Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr \
    --rpc-url https://api.mainnet-beta.solana.com \
    --program-id 6YbCoGRzG1K9PouQKNrs3bMnHye77FsiWCTLYnU59TRU \
    --keypair-path /path/to/claimer-keypair.json \
    claim \
    --merkle-tree-path merkle_trees/tree_5.json
```

-   `--keypair-path`: Path to the keypair of the person claiming the tokens.
-   `--merkle-tree-path`: Path to the specific merkle tree JSON file for this airdrop version.

## Step 6: Test Clawback Functionality

After the clawback period has started (at least 1 day after the vesting period ends), anyone can trigger the clawback for unclaimed tokens. The funds will be sent to the pre-configured `clawback_receiver`.

### Prerequisites for Clawback
- The clawback period must have started (`clawback_start_ts` must be in the past).
- The distributor must have unclaimed tokens.
- The distributor must not have already been clawed back.

### Execute Clawback

```bash
source ~/.bashrc && cargo run --bin cli -- \
    --mint Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr \
    --rpc-url https://api.mainnet-beta.solana.com \
    --program-id 6YbCoGRzG1K9PouQKNrs3bMnHye77FsiWCTLYnU59TRU \
    --keypair-path /path/to/any/funded/keypair.json \
    clawback \
    --merkle-tree-path merkle_trees \
    --airdrop-version 5
```

-   `--merkle-tree-path`: Directory containing the merkle tree JSON file for the specified airdrop version.
-   `--airdrop-version`: The version of the airdrop to claw back.
-   `--keypair-path`: Path to any funded keypair to pay for the transaction. This does not need to be the admin.

### Example Clawback Command

```bash
source ~/.bashrc && cargo run --bin cli -- \
    --mint Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr \
    --rpc-url https://api.mainnet-beta.solana.com \
    --program-id 6YbCoGRzG1K9PouQKNrs3bMnHye77FsiWCTLYnU59TRU \
    --keypair-path ~/.config/solana/id.json \
    clawback \
    --merkle-tree-path merkle_trees \
    --airdrop-version 7
```

### Verify Clawback Success

After executing the clawback, you can verify it was successful by:

1. **Check the distributor status** via the API:
   ```bash
   curl -s http://127.0.0.1:7002/distributors | jq '.[] | select(.pubkey == "<DISTRIBUTOR_PDA>") | {clawed_back: .clawed_back, total_amount_claimed: .total_amount_claimed}'
   ```

2. **Check the clawback receiver's token balance** to confirm it received the clawed back tokens.

### Important Notes

-   **Clawback Timing**: The clawback can only be executed after the `clawback_start_ts` timestamp has passed.
-   **One-time Operation**: Clawback can only be executed once per distributor.
-   **Permissionless**: Anyone can trigger the clawback instruction; it is not restricted to the admin. The transaction simply needs a keypair to pay for the fees.
-   **Unclaimed Tokens**: Only unclaimed tokens will be clawed back; already claimed tokens remain with the claimants.

## Step 7: Emergency Withdraw with Close Distributor

This is an admin-only action for emergency situations to withdraw all remaining funds from the distributor's token vault. This action is only possible if the distributor was created with the `--closable` flag.

```bash
source ~/.bashrc && cargo run --bin cli -- \
    --mint Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr \
    --rpc-url https://api.mainnet-beta.solana.com \
    --program-id 6YbCoGRzG1K9PouQKNrs3bMnHye77FsiWCTLYnU59TRU \
    --keypair-path ~/.config/solana/id.json \
    close-distributor \
    --merkle-tree-path merkle_trees \
    --airdrop-version 5 \
    --destination-token-account <DESTINATION_TOKEN_ACCOUNT_PUBKEY>
```

-   `--keypair-path`: Path to the admin keypair for this distributor. **Must be the admin.**
-   `--merkle-tree-path`: Directory containing the merkle tree JSON file for the specified airdrop version.
-   `--airdrop-version`: The version of the airdrop to close.
-   `--destination-token-account` (Optional): The token account where funds will be sent. If not provided, funds will be sent to the admin's associated token account.

## Key Differences: Clawback vs. Close Distributor

| Feature                   | `clawback`                                                      | `close-distributor`                                           |
| ------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------- |
| **Permission**            | Anyone can execute (permissionless).                            | Only the distributor's `admin` can execute.                   |
| **Timing**                | Only after `clawback_start_ts`.                                 | Any time.                                                     |
| **Requirement**           | Distributor must not have been clawed back.                     | Distributor must have been created with the `closable` flag.  |
| **Funds Destination**     | The `clawback_receiver` set during the distributor's creation.  | The `destination_token_account` specified in the command.     |
| **Account State**         | The `clawed_back` flag on the distributor is set to `true`.       | The on-chain distributor account is closed and lamports are recovered. |


```
cargo run --bin cli -- create-merkle-tree \
    --csv-path recipients.csv \
    --merkle-tree-path merkle_trees \
    --max-nodes-per-tree 1000000 \
    --decimals 9 \
    --start-airdrop-version 11 \
    --amount 0
```

```
current_ts=$(date +%s)

# Calculate future timestamps
start_vesting_ts=$((current_ts + 120)) # 2 minutes from now
end_vesting_ts=$((start_vesting_ts + 1)) # 1 second after start for immediate release
clawback_start_ts=$((end_vesting_ts + 3 * 7 * 24 * 60 * 60)) # 3 weeks after end

echo "Calculated Timestamps:"
echo "Start Vesting TS: $start_vesting_ts"
echo "End Vesting TS:   $end_vesting_ts"
echo "Clawback Start TS: $clawback_start_ts"

# Store timestamps in environment variables for later use
export START_TS=$start_vesting_ts
export END_TS=$end_vesting_ts
export CLAWBACK_TS=$clawback_start_ts
```
```
source ~/.bashrc && cargo run --bin cli -- \
    --mint 9v6BKHg8WWKBPTGqLFQz87RxyaHHDygx8SnZEbBFmns2 \
    --rpc-url https://api.mainnet-beta.solana.com \
    --program-id 6YbCoGRzG1K9PouQKNrs3bMnHye77FsiWCTLYnU59TRU \
    --keypair-path ~/.config/solana/id.json \
    new-distributor \
    --merkle-tree-path merkle_trees \
    --start-vesting-ts $START_TS \
    --end-vesting-ts $END_TS \
    --clawback-start-ts $CLAWBACK_TS \
    --clawback-receiver-owner 6dfrn1uceBjjUi6HMsJimgcC4qaGbPQQtsHC1Wtwhct1 \
    --enable-slot 0 \
    --closable \
    --airdrop-version 11
```
```
spl-token transfer 9v6BKHg8WWKBPTGqLFQz87RxyaHHDygx8SnZEbBFmns2 74999 HJevtWp2Phi3RXvqmBtSSDW4LmFqQixogmK6sg9ifWo6 --owner ~/.config/solana/id.json --fund-recipient --url https://api.mainnet-beta.solana.com --allow-non-system-account-recipient
```