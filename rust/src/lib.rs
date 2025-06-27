#[cfg(test)]
mod tests {
    // use super::*;
    use solana_sdk::{ signature::{ Keypair, Signer }, pubkey::Pubkey };
    use bs58;
    use std::io::{ self, BufRead };

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as a base58 string: ");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file format is :");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a JSON byte array (e.g [12, 33,...]):");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        println!("Your base58-encoded private key is: ");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string());
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file: ");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn airdrop() {}

    #[test]
    fn transfer_sol() {}
}
