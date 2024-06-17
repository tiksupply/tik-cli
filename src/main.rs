mod mine;
mod register;
mod rewards;
mod claim;
mod conf;
use clap::{command, Parser, Subcommand};
use sui_types::base_types::{SequenceNumber, SuiAddress};
use sui_types::{base_types::ObjectID, transaction::ObjectArg};
use std::path::Path;
use std::process::Command;
use sui_sdk::{ SuiClient, SuiClientBuilder};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};


#[derive(Clone)]
struct Miner {
    pub keypair_filepath: String,
    pub sui_client: SuiClient,
    pub lock_days:u8,
    pub address:SuiAddress,
    pub treasury_obj:ObjectArg,
    pub miner_obj:ObjectArg,
    pub epochs_obj:ObjectArg,
    pub clock_obj:ObjectArg,
    pub current_epoch:u64,
    pub testnet:bool

}
#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    #[arg(
        long,
        value_name = "NETWORK_URL",
        help = "Network address of your RPC provider",
        default_value = "https://fullnode.mainnet.sui.io:443",
        global = false
    )]
    rpc: String,

    #[arg(
        long,
        value_name = "KEYPAIR_FILEPATH",
        help = "Filepath to the keypair to use.",
        default_value = "",
        global = false
    )]
    keypair: String,

    #[arg(
        long,
        help = "For testnet",
        default_value = "false",
        global = false
    )]
    testnet: bool,

  

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Start mining")]
    Mine(MineArgs),
    #[command(about = "Check how much TIK you've earned.")]
    Rewards,
    #[command(about = "Claim rewards to your wallet.")]
    Claim,
    #[command(about = "Generate a keypair.")]
    Gen,
    #[command(about = "Show the suiprivate key from a keypair.")]
    Prikey,
    #[command(about = "Import your suiprivate key to create a keypair.")]
    Import {
        #[arg(short, long)]
        key: String,
    },
}


#[derive(Parser, Debug)]
struct MineArgs {
    #[arg(
        long,
        short,
        value_name = "Lock days",
        help = "The number of days you want to lock the rewards. Each day of locking increases the share by 1.",
        default_value = "0"
    )]
    lock: u8,
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

  
    let args = Args::parse();
    let default_keypair = args.keypair;
    let cloned_keypair = default_keypair.clone(); 
    let pathstring=cloned_keypair.to_lowercase();
    let path = Path::new(&pathstring);
    match args.command {Commands::Gen=>{conf::gen_newkey();return Ok(());}
    Commands::Mine(_) =>{},
    Commands::Rewards => {},
    Commands::Claim => {}, 
    Commands::Prikey => {

        conf::show_prikey(path);

        return Ok(());
    },
    Commands::Import { key } => {

        conf::import_newkey(key);

        return Ok(());
    }
    }

  
    let rpc = args.rpc;
    let test=args.testnet;



    let sui_client =SuiClientBuilder::default().build(rpc.clone()).await?;

  
    let kp =  conf::read_keypair_from_file(path).expect("Cannot load keypair. Please ensure the --keypair parameter path is correct.");
    let address: SuiAddress = (&kp.public()).into();
    let mut keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;

    let _ = Command::new("cmd").args(&["/c", "cls"]).status();
    println!("");
    println!("Welcome to TIK.supply!");
    println!("Addr:{:?}",address);
   
   let getkey =keystore.get_key(&address);
   match getkey  {  
        Ok(_keypar)=>{}
        Err(_err)=>
        {
            let rst=  keystore.add_key(Some("x".to_owned()+&address.to_string()),kp.copy());
            match rst {
                Ok(_alias) => {
                }
                Err(_err) => {
                    println!("Error creating alias: {}", _err);
                }
            }
            let _ = keystore.save();
        }        
   }

   
    let mut miner = Miner::new(
        sui_client.clone(),
        default_keypair,
        address,
        ObjectArg::SharedObject { id: (ObjectID::random()), initial_shared_version: (0.into()), mutable: (true) },
        ObjectArg::SharedObject { id: (ObjectID::random()), initial_shared_version: (0.into()), mutable: (true) },
        ObjectArg::SharedObject { id: (ObjectID::random()), initial_shared_version: (0.into()), mutable: (true) },
        ObjectArg::SharedObject { id: (ObjectID::random()), initial_shared_version: (0.into()), mutable: (true) },
        0,
        test

    );

    let treasury_oid=ObjectID::from_hex_literal(&miner.get_treasury_id()).expect("Failed to parse ObjectID");
    let miner_oid = ObjectID::from_hex_literal(&miner.get_miner_id()).expect("Failed to parse ObjectID");
    let epochs_oid=ObjectID::from_hex_literal(&miner.get_epochs_id()).expect("Failed to parse ObjectID");
    let clock_oid=ObjectID::from_hex_literal("0x6").expect("Failed to parse ObjectID");
    let initial_shared_version=SequenceNumber::from(miner.get_init_ver().to_owned());
    let clocok_shared_version=SequenceNumber::from(1);
    miner.treasury_obj=ObjectArg::SharedObject {
        id: treasury_oid,
        initial_shared_version,
        mutable: true,
    };
    miner.miner_obj=ObjectArg::SharedObject {
        id: miner_oid,
        initial_shared_version,
        mutable: true,
    };
    miner.epochs_obj=ObjectArg::SharedObject {
        id: epochs_oid,
        initial_shared_version,
        mutable: true,
    };
    miner.clock_obj=ObjectArg::SharedObject {
        id: clock_oid,
        initial_shared_version:clocok_shared_version,
        mutable: false,
    };
    miner.register().await?;

    println!("---------------Balance------------------");

    let suibalance=conf::get_coinbalance(&sui_client, &address,None,9.0);
    let tikbalance=conf::get_coinbalance(&sui_client, &address,Some(miner.get_coin_type().to_string()) ,12.0);
    println!("{:?} TIK",tikbalance.await?);
    println!("{:?} SUI",suibalance.await?);
    println!("----------------------------------------");
   

    match args.command {
        Commands::Mine(args) => {
            Miner::send_windata(miner.clone(),args.lock as u64).await;
            let _ = miner.mine(args.lock).await;
        }
        Commands::Rewards => {
            miner.rewards().await;
        }
        Commands::Claim => {
            claim::claim(miner.clone()).await;
        }
        Commands::Gen => {
           
        }
        Commands::Prikey => {
           
        }, 
        Commands::Import { key } => {
            drop(key);
        }
    }

    Ok(())
}






impl Miner {
    pub fn new(sui_client :SuiClient , keypair_filepath: String, address:SuiAddress,treasury_obj:ObjectArg,miner_obj:ObjectArg ,epochs_obj:ObjectArg,clock_obj:ObjectArg,current_epoch:u64,testnet:bool) -> Self {
        Self {
            sui_client,
            keypair_filepath,
            lock_days: 0,
            address,
            treasury_obj,
            miner_obj,
            epochs_obj,
            clock_obj,
            current_epoch,
            testnet
        }
    }

   
}
