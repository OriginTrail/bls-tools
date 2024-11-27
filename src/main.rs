use clap::{Parser, Subcommand};
use sylow::{KeyPair, Fp, G1Projective, G2Projective, G1Affine, G2Affine, GroupTrait, pairing, XMDExpander};
use serde_json::json;
use hex;
use sha3::Keccak256;

const DST: &[u8; 30] = b"WARLOCK-CHAOS-V01-CS01-SHA-256";
const SECURITY_BITS: u64 = 128;

#[derive(Parser)]
#[command(name = "BLS Tool")]
#[command(version = "1.0")]
#[command(about = "Tool for BLS key generation, signing, and aggregation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenerateKeys,
    Sign {
        #[arg(short, long)]
        secret: String,

        #[arg(short, long)]
        message: String,
    },
    PublicKeyFromSecret {
        #[arg(short, long)]
        secret: String,
    },
    AggregateKeys {
        #[arg(short, long, num_args=1..)]
        public_keys: Vec<String>,
    },
    AggregateSignatures {
        #[arg(short, long, num_args=1..)]
        signatures: Vec<String>,
    },
    Verify {
        #[arg(short, long)]
        signature: String,

        #[arg(short, long)]
        public_key: String,

        #[arg(short, long)]
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKeys => {
            let key_pair = KeyPair::generate();
            let result = json!({
                "secretKey": hex::encode(key_pair.secret_key.to_be_bytes()),
                "publicKey": hex::encode(G2Affine::from(key_pair.public_key).to_be_bytes()),
            });
            println!("{}", result);
        }
        Commands::PublicKeyFromSecret { secret } => {
            let secret_key_bytes = hex::decode(secret).expect("Invalid hex in secret key");
            let secret_key_array: [u8; 32] = secret_key_bytes
                .try_into()
                .expect("Secret key must be 32 bytes");
            let secret_key = Fp::from_be_bytes(&secret_key_array)
                .expect("Failed to deserialize secret key");
            
            let public_key = G2Projective::generator() * secret_key;
            let public_key_affine = G2Affine::from(public_key);
            let public_key_bytes = public_key_affine.to_be_bytes();

            println!("{}", hex::encode(public_key_bytes));
        }
        Commands::Sign { secret, message } => {
            let secret_key_bytes = hex::decode(secret).expect("Invalid hex in secret key");
            let secret_key_array: [u8; 32] = secret_key_bytes
                .try_into()
                .expect("Secret key must be 32 bytes");
            let secret_key = Fp::from_be_bytes(&secret_key_array)
                .expect("Failed to deserialize secret key");
            let expander = XMDExpander::<Keccak256>::new(DST, SECURITY_BITS);
            let hashed_message = G1Projective::hash_to_curve(&expander, message.as_bytes())
                .expect("Hashing failed");
            let signature = hashed_message * secret_key;
            println!("{}", hex::encode(G1Affine::from(signature).to_be_bytes()));
        }
        Commands::AggregateKeys { public_keys } => {
            let mut agg_key = G2Projective::zero();
            for key_hex in public_keys {
                let key_bytes = hex::decode(key_hex).expect("Invalid hex in public key");
                let key_array: [u8; 128] = key_bytes
                    .try_into()
                    .expect("Public key must be 128 bytes");
                let pubkey_affine = G2Affine::from_be_bytes(&key_array)
                    .into_option()
                    .expect("Invalid public key");
                let pubkey = G2Projective::from(pubkey_affine);
                agg_key = agg_key + pubkey;
            }
            println!("{}", hex::encode(G2Affine::from(agg_key).to_be_bytes()));
        }
        Commands::AggregateSignatures { signatures } => {
            let mut agg_sig = G1Projective::zero();
            for sig_hex in signatures {
                let sig_bytes = hex::decode(sig_hex).expect("Invalid hex in signature");
                let sig_array: [u8; 64] = sig_bytes
                    .try_into()
                    .expect("Signature must be 64 bytes");
                let sig_affine = G1Affine::from_be_bytes(&sig_array)
                    .into_option()
                    .expect("Invalid signature");
                let sig = G1Projective::from(sig_affine);
                agg_sig = agg_sig + sig;
            }
            println!("{}", hex::encode(G1Affine::from(agg_sig).to_be_bytes()));
        }
        Commands::Verify {
            signature,
            public_key,
            message,
        } => {
            let sig_bytes = hex::decode(signature).expect("Invalid hex in signature");
            let sig_array: [u8; 64] = sig_bytes
                    .try_into()
                    .expect("Signature must be 64 bytes");
            let agg_signature_affine = G1Affine::from_be_bytes(&sig_array)
                .into_option()
                .expect("Invalid signature");
            let agg_signature = G1Projective::from(agg_signature_affine);

            let key_bytes = hex::decode(public_key).expect("Invalid hex in public key");
            let key_array: [u8; 128] = key_bytes
                    .try_into()
                    .expect("Public key must be 128 bytes");
            let agg_pubkey_affine = G2Affine::from_be_bytes(&key_array)
                .into_option()
                .expect("Invalid public key");
            let agg_pubkey = G2Projective::from(agg_pubkey_affine);

            let expander = XMDExpander::<Keccak256>::new(DST, SECURITY_BITS);
            let hashed_message = G1Projective::hash_to_curve(&expander, message.as_bytes())
                .expect("Hashing failed");

            let lhs = pairing(&agg_signature, &G2Projective::generator());
            let rhs = pairing(&hashed_message, &agg_pubkey);

            println!("{}", json!({ "valid": lhs == rhs }));
        }
    }
}
