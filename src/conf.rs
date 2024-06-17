use std::sync::atomic::AtomicU64;
use sui_keys::{key_derive::generate_new_key, keypair_file::write_keypair_to_file};
use once_cell::sync::Lazy;
use sui_json_rpc_types::{CheckpointId, SuiMoveStruct, SuiMoveValue, SuiParsedData};
use sui_sdk::{
    rpc_types::{SuiObjectData, SuiObjectDataOptions},
    SuiClient,
};
use sui_types::{base_types::{ObjectID, SequenceNumber, SuiAddress}, crypto::{EncodeDecodeBase64, SignatureScheme, SuiKeyPair}, digests::ObjectDigest, error::SuiError};
use tokio::sync::Mutex;

use crate::Miner;
pub struct OnlineData {
    package_id: &'static str,
    treasury_id: &'static str,
    miner_id: &'static str,
    epochs_id: &'static str,
    coin_type: &'static str,
    init_ver:&'static u64,
}

impl OnlineData {
    pub fn package_id(&self) -> &'static str {
        self.package_id
    }
    pub fn treasury_id(&self) -> &'static str {
        self.treasury_id
    }
    pub fn miner_id(&self) -> &'static str {
        self.miner_id
    }
    pub fn epochs_id(&self) -> &'static str {
        self.epochs_id
    }
    pub fn coin_type(&self) -> &'static str {
        self.coin_type
    }
    pub fn init_ver(&self) -> &'static u64 {
        self.init_ver
    }

}

pub const  TESTNET_DATA: OnlineData = OnlineData {
    package_id: "0xaba2be6103e8d38d38173648e527d0ecf20adf3ad989905f549cf7332fd5e7a1",
    treasury_id: "0x83ba2a98df2ca8c4f07710c2d8ecb54d7397ee24a7f48036b8b053d3bcccd899",
    miner_id: "0x4de8ffe8958bb8b6a1cc4d3100feb34fd0b380a00ca2ec9b6356fe1636b88d46",
    epochs_id: "0x14fa8410656a176ed1e67f2dd4339e9e498dcb1cff76f2c399b780335bbb3f3e",
    coin_type:"0xaba2be6103e8d38d38173648e527d0ecf20adf3ad989905f549cf7332fd5e7a1::tikcoin::TIKCOIN",
    init_ver:&34052830,
};


pub const  MAINNET_DATA: OnlineData = OnlineData {
    package_id: "0xaba2be6103e8d38d38173648e527d0ecf20adf3ad989905f549cf7332fd5e7a1",
    treasury_id: "0x77e2af16400ddf8d693fa34da03c2c9eaba8a2754191fac397d748aaf56cae07",
    miner_id: "0x3219c0db90e57fb9d5be377d34302170e9092628bdeaefefe3fbe64f93099664",
    epochs_id: "0x643db1d976171ecf577155e8258f89f7d1e9013e2bb468f1918703ea6426705f",
    coin_type:"0xb85d3853c7a33cf6cc243d5ae748b3a85ccc0ea2142db680aeedfa079b19fe13::tikcoin::TIKCOIN",
    init_ver:&106971063,
};

pub const EPOCH_REWARD:f64=0.000277_777_778;
pub const BASE_SHARE:u8=10;
pub const MINER_SHARE:f64=0.99;

pub  static mut IS_CLAIMING: bool=false;
pub static REWARD_IT_COUNT: AtomicU64 = AtomicU64::new(0);


pub static WIN_DATA: Lazy<Mutex<Vec<u64>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub async fn add_to_win_data(value: u64) {
    let mut data = WIN_DATA.lock().await;
    data.push(value);
}

pub async fn get_win_data() -> Vec<u64> {
    let data = WIN_DATA.lock().await;
    data.clone() 
}

pub async fn remove_from_win_data(value: u64) -> bool {
    let mut data = WIN_DATA.lock().await;
    if let Some(pos) = data.iter().position(|&x| x == value) {
        data.remove(pos);
        true
    } else {
        false
    }
}

pub async fn set_current_epoch(miner: &mut Miner) -> Result<u64, SuiError> {
    let last_check_point_result = miner.sui_client.read_api().get_latest_checkpoint_sequence_number().await;

    match last_check_point_result {
        Ok(last_check_point) => {
            let checkpoint_result = miner.sui_client.read_api().get_checkpoint(CheckpointId::SequenceNumber(last_check_point)).await;

            match checkpoint_result {
                Ok(checkpoint) => {
                    // Extract and return the epoch
                    return Ok(checkpoint.timestamp_ms / 1000);
                }
                Err(_error) => {
                    // Handle checkpoint retrieval error
                    return Err("Network Error".into());
                }
            }
        }
        Err(_error) => {
            // Handle latest checkpoint sequence number retrieval error
            return Err("Network Error".into());
        }
    }
}

pub async fn get_genesis( miner:&Miner ) -> u64 {
    let opt=SuiObjectDataOptions::new();
    let rst=miner.sui_client.read_api().get_object_with_options(ObjectID::from_hex_literal(miner.get_miner_id()).expect("Failed to parse ObjectID"), opt.with_content()).await;
    let rstdata=  rst.unwrap().data.unwrap();
    return  get_genesis_value(rstdata);
}

fn get_genesis_value( suiobjdata:SuiObjectData ) -> u64 {
    let mut rst:u64=0;
    if let Some( SuiParsedData::MoveObject(move_object)) =  suiobjdata.content {

        if let SuiMoveStruct::WithFields(fields) = move_object.fields {
            
            if let Some(gen) = fields.get("Genesis") {
                match gen {
                    SuiMoveValue::String(js) => {
                        rst=js.parse().unwrap();      
                    },
                    _ => {
                 
                    }
                }
            }
        }

    }
    rst
}



impl Miner {
   
    pub fn get_package_id(&self)-> &'static str
   {
      if self.testnet
      {
        return  TESTNET_DATA.package_id();
      }
      else {
        return  MAINNET_DATA.package_id();
      }
   }
   pub fn get_treasury_id(&self)-> &'static str
   {
      if self.testnet
      {
        return  TESTNET_DATA.treasury_id();
      }
      else {
        return  MAINNET_DATA.treasury_id();
      }
   }
   pub fn get_miner_id(&self)-> &'static str
   {
      if self.testnet
      {
        return  TESTNET_DATA.miner_id();
      }
      else {
        return  MAINNET_DATA.miner_id();
      }
      
   }
   pub fn get_epochs_id(&self)-> &'static str
   {
      if self.testnet
      {
        return  TESTNET_DATA.epochs_id();
      }
      else {
        return  MAINNET_DATA.epochs_id();
      }
   }
   pub fn get_coin_type(&self)-> &'static str
   {
      if self.testnet
      {
        return  TESTNET_DATA.coin_type();
      }
      else {
        return  MAINNET_DATA.coin_type();
      }
   }
   pub fn get_init_ver(&self)-> &'static u64
   {
      if self.testnet
      {
        return  TESTNET_DATA.init_ver();
      }
      else {
        return  MAINNET_DATA.init_ver();
      }
   }

   


}
pub async fn get_coinbalance(rpc_client: &SuiClient,sender: &SuiAddress, cointype:Option<String>, decimals:f64) -> Result<f64, anyhow::Error>
{
    let balance = rpc_client
    .coin_read_api()
    .get_balance(sender.to_owned(), cointype)
    .await?;

  

let decif64 = 10.0_f64.powf(decimals);
    return   Ok(balance.total_balance as f64/decif64);
}
pub async fn fetch_sorted_gas_coins(
    rpc_client: &SuiClient,
    sender: &SuiAddress,
) -> anyhow::Result<Vec<(ObjectID, SequenceNumber, ObjectDigest)>> {


        let coins = rpc_client
        .coin_read_api()
        .get_coins(  sender.to_owned(), None, None, None)
        .await.unwrap();
        let gascoin = coins.data;
        let mut gasbalance=0;
        let mut coins:Vec<(ObjectID, SequenceNumber, ObjectDigest)>=vec![];
        for coin in gascoin.iter() {
            gasbalance=gasbalance+coin.balance;
            coins.push(coin.object_ref());

        }

    Ok(coins)
}



pub fn gen_newkey(){
    let (sui_address, skp, _scheme, _phrase) =
      generate_new_key(SignatureScheme::ED25519, None, None).expect("err");
     let file = format!("{sui_address}.key");
      write_keypair_to_file(&skp, file.clone()).expect("err");
      println!("Generated! {}",file);
      println!("TO mine:    --keypair {}  mine",file);
      println!("Show privatekey: --keypair  {} prikey",file);
}
pub fn show_prikey<P: AsRef<std::path::Path>>(path: P) {  
    let keypair = read_keypair_from_file(path).expect("Cannot load keypair. Please ensure the --keypair parameter path is correct.");
    println!("Privatekey: {}",keypair.encode().unwrap());  
}

pub fn import_newkey(key:String){
    let kp= SuiKeyPair::decode(&key).unwrap();
   
     let sui_address=  SuiAddress::from(&kp.public()).to_string();

      let file = format!("{sui_address}.key");
      write_keypair_to_file(&kp, file.clone()).expect("err");
      println!("Imported! {}",file);
      println!("TO mine:    --keypair {}  mine",file);
      println!("Show privatekey: --keypair  {} prikey",file);
}

pub fn read_keypair_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<SuiKeyPair> {
    let contents = std::fs::read_to_string(path)?;
    let keypair = SuiKeyPair::decode_base64(contents.as_str().trim())?;
    Ok(keypair)
}
