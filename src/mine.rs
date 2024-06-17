use crate::claim;
use crate::conf;
use crate::Miner;



use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::vec;



use fastcrypto::hash::Blake2b256;
use fastcrypto::hash::HashFunction;
use shared_crypto::intent::Intent;
use sui_config::sui_config_dir;
use sui_config::SUI_KEYSTORE_FILENAME;
use sui_json_rpc_types::SuiMoveStruct;
use sui_json_rpc_types::SuiMoveValue;
use sui_json_rpc_types::SuiTransactionBlockResponseOptions;
use sui_json_rpc_types::SuiParsedData;
use sui_keys::keystore::AccountKeystore;
use sui_keys::keystore::FileBasedKeystore;
use sui_types::base_types::ObjectID;
use sui_types::base_types::SequenceNumber;
use sui_types::digests::ObjectDigest;
use sui_types::dynamic_field::DynamicFieldName;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;

use sui_types::transaction::Transaction;
use sui_types::transaction::TransactionData;
use sui_types::Identifier;
use sui_types::TypeTag;
use sui_types::base_types::SuiAddress;

use termcolor::{ ColorChoice, StandardStream};
use tokio::time::Instant;


impl Miner {

    pub async fn mine(&mut self, lock_days: u8)-> anyhow::Result<()> {
      
        
        self.lock_days=lock_days;
        println!("");
        let pathstring=&self.keypair_filepath.to_lowercase();
        let path = Path::new(pathstring);
        let kp = conf::read_keypair_from_file(path)?;
        let _address: SuiAddress = (&kp.public()).into();
       
        println!("--------------------- Base share: {} Lock days: {} Your share: {} ----------------------",conf::BASE_SHARE,lock_days,conf::BASE_SHARE+lock_days);
        println!("Everything looks great! Let's Tik !");
        thread::sleep(Duration::from_secs(2));
        let _ = self.set_current_epoch().await;
        let get_epochs_id=self.get_epochs_id();
      
      
        let _miner_obj=self.miner_obj;
        let _epochs_obj=self.epochs_obj;
        let _clock_obj=self.clock_obj;
        let miner_clone=self.clone();
        let  genesis=conf::get_genesis(&miner_clone).await;
       
      
        loop {
          
            let current_epoch_clone = self.current_epoch.clone(); // Clone current_epoch

            if genesis>current_epoch_clone{

                let  cont=genesis -current_epoch_clone ;

                println!("\x1b[91m●\x1b[0m Mining has not started yet. Please wait for {:?} secend. ", cont);
                self.current_epoch=self.current_epoch+1;
                thread::sleep(Duration::from_secs(1));
                continue;

            }
           
            if unsafe { conf::IS_CLAIMING } ==false{
            
                let address_clone=self.address.clone();
                let sui_client_clone = self.sui_client.clone(); // Clone suiClient for use inside the async block
                let mut _stdout = StandardStream::stdout(ColorChoice::Always);
         

                let mut random_vector: Vec<u8> = Vec::new();
                random_vector.append(&mut address_clone.to_vec());
                random_vector.append(&mut  Self::u64_to_ascii(current_epoch_clone));
                let mut hash = Blake2b256::default();
                hash.update(&random_vector);
                let hash_result = hash.finalize();

                let hash_result_bytes = hash_result.as_ref();
                let mut result: u128 = 0;
                for &num in hash_result_bytes.iter() {
                    result = (result << 8) | num as u128;
                }
                tokio::spawn(async move {  
                //get diff
                let start = Instant::now();
                let mut diff: u128=24;
                let diff_epoch=current_epoch_clone-30;
                let parent_object_id = ObjectID::from_hex_literal(get_epochs_id).expect("Failed to parse ObjectID");
                let dynamic_name = DynamicFieldName {
                    type_: TypeTag::U64,
                    value:  serde_json::json!(diff_epoch.to_string())  ,
                };
                let dynamic_fields: Result<sui_json_rpc_types::SuiObjectResponse, sui_sdk::error::Error> = sui_client_clone.read_api()
                .get_dynamic_field_object(parent_object_id, dynamic_name.clone() )
                .await;

                match dynamic_fields {
                    std::result::Result::Ok(sui_object_response) => {
                        if let Some(sui_object_data) = sui_object_response.data {
                            if let Some(content) = sui_object_data.content {
                                if let SuiParsedData::MoveObject(move_object) = content {
                                    if let SuiMoveStruct::WithFields(fields) = move_object.fields {
                                        if let Some(shares_miners) = fields.get("shares_miners") {
                                            if let SuiMoveValue::Vector(shares_miners_vector) = shares_miners {
                                                if let SuiMoveValue::String(share_string) = &shares_miners_vector[1] {

                                                    match share_string.parse::<u128>() {
     
                                                            Ok(value) => {
                                                                 diff=diff+value/1000
                                                            },
                                                            Err(_e) => {
                                                                println!("Failed to parse string to u128: {:?}", _e);
                                                            }

                                                    }

                                                }
                                              
                                            }
                                        }
                                    }

                                }
                            }
                        }else {
                               let value=0;
                               diff=diff+value/1000
                        }
                    },
                    std::result::Result::Err(_e) => {
                        eprintln!("Error fetching dynamic field object: {:?}", _e);
                    }
                }


                let _duration = start.elapsed();
               
                    if result % diff==0
                    {
                        if _duration.as_secs_f64()<10.0 {
                            conf::add_to_win_data(current_epoch_clone).await;
                            println!("  \x1b[93m\u{2714}\x1b[0m  Epoch: {:?}  pass: {:?}   WIN!!!     Submitting....   ", current_epoch_clone,result);
                        }else {
                            println!("-------------------------------------------------------------------------------");
                            println!("  \x1b[91m●\x1b[0m Epoch: {:?}   pass: {:?} TimeOut...   ", current_epoch_clone,result);
                            println!("-------------------------------------------------------------------------------");
                        }
                   
                    }else {
                        println!("      Epoch: {:?}  pass: {:?}  Nothing... ", current_epoch_clone,result);
                    }
             

            });


        }
          
            self.current_epoch=self.current_epoch+1;
           
            thread::sleep(Duration::from_secs(1));
        }
        
    }
    pub async fn send_windata(mut  self:Miner,lockdays:u64)
    {
        tokio::spawn(async move {  
        let packageid = self.get_package_id();
        let package_id = ObjectID::from_hex_literal(&packageid).expect("Failed to parse ObjectID");
     
        let keystore = FileBasedKeystore::new(&sui_config_dir().unwrap().join(SUI_KEYSTORE_FILENAME)).unwrap();
      
        loop {

            let win_data=conf::get_win_data().await;
            let length = win_data.len();
            let mut gas_budget = 0;
         
            if length>0 {
                let mut pt_builder = ProgrammableTransactionBuilder::new();
                for i in (0..length).rev() {

                    let miner_o=pt_builder.obj (self.miner_obj).expect("Err minerId");
                    let epochs_o=pt_builder.obj(self.epochs_obj).expect("Err pochsId");
                    let clock_o=pt_builder.obj(self.clock_obj).expect("Err clockId");
                    let current_epoch_arg=pt_builder.pure(win_data[i] as u64).expect("Err current_epochId");
                    let lockdays_arg=pt_builder.pure(lockdays as u64).expect("Err lockdaysId");
                    let mine_mod = Identifier::from_str("mine").expect("err mine");
                    let mine_function = Identifier::from_str("mine").expect("err mine_function"); 
                    pt_builder.programmable_move_call(
                        package_id,
                        mine_mod,
                        mine_function,
                        vec![],
                        vec![miner_o,epochs_o,current_epoch_arg,lockdays_arg,clock_o],
                    );
                    conf::remove_from_win_data(win_data[i]).await;
                    gas_budget = gas_budget+ 12_000_000;
                }

                let pt = pt_builder.finish();

               
            let coins = self.sui_client
            .coin_read_api()
            .get_coins(  self.address, None, None, None)
            .await.unwrap();
        let gascoin = coins.data;
        let mut gasbalance=0;
        let mut coins:Vec<(ObjectID, SequenceNumber, ObjectDigest)>=vec![];
        for coin in gascoin.iter() {
            gasbalance=gasbalance+coin.balance;
            coins.push(coin.object_ref());

        }

        if gasbalance<(gas_budget as u64)
        {
              println!("-----------------------------------------------------------------------------------");
              println!("  \x1b[91m●\x1b[0m Error! Not enough balance for gas fee!  balance: {:?} SUI need:{:?} SUI", gasbalance,(gas_budget));
              println!("-----------------------------------------------------------------------------------");
              continue;
        }

                    let gas_price = self.sui_client.read_api().get_reference_gas_price().await.unwrap();
                    let tx_data = 
                    TransactionData::new_programmable( self.address,coins,pt,gas_budget,gas_price);
                    let signature = keystore.sign_secure(&self.address, &tx_data, Intent::sui_transaction()).unwrap();

                    let mut trans_lock=SuiTransactionBlockResponseOptions::new();
                    trans_lock.show_events=true;

                    let transaction_response = self.sui_client
                    .quorum_driver_api()
                    .execute_transaction_block(
                        Transaction::from_data(tx_data, vec![signature]),
                        trans_lock.with_events(),
                        Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                    )
                    .await;
                match transaction_response {
                    Ok(response) => {
                        match response.events {
                            Some(events) => {
                                for event in events.data.iter() {
                                    if event.type_.name.as_str() == "RstEvet" {
                                        let epoch = event.parsed_json["epoch"].as_str().unwrap_or("");
                                        let mut share = event.parsed_json["share"].as_str().unwrap_or("").to_string();
                                        let _ext_share = event.parsed_json["ext_share"].as_str().unwrap_or("");
            
                                        if _ext_share != "0" {
                                            share = format!("{} (+{} Ext share)", share, _ext_share);
                                        }
            
                                        let unlockstr = event.parsed_json["unlock"].as_str().unwrap_or("");
                                        let unlock: i64 = unlockstr.parse().unwrap_or(0);
            
                                        let mut bytes = Vec::new();
                                        if let Some(tmp) = event.parsed_json["euid"].as_array() {
                                            for number in tmp {
                                                if let Some(byte) = number.as_u64() {
                                                    bytes.push(byte as u8);
                                                } else {
                                                    println!("Error: Unable to convert number to u8");
                                                }
                                            }
                                        }
            
                                        println!(
                                            "  \x1b[92m\u{2714}\x1b[0m  Submitted ---->  Epoch: {}  Share: {}  unlock: {} Days",
                                            epoch, share, unlock
                                        );
                                        let cot =   conf::REWARD_IT_COUNT.fetch_add(1, Ordering::SeqCst);
       
                                        if cot>=98
                                        {   
                                            println!(" ----------------  Start claim --------------------- ");
                                            unsafe { conf::IS_CLAIMING = true };
                                            conf::REWARD_IT_COUNT.store(0, Ordering::SeqCst);
                                            claim::claim(self.clone()).await;
                                        }
                                    }
                                }
                           
                                if events.data.is_empty() {
                                    Self::timeout_err(&mut self).await;
                                    
                                }
                            }
                            None => {
                                Self::timeout_err(&mut self).await;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        Self::timeout_err(&mut self).await;
                    }
                }
        
         }

            thread::sleep(Duration::from_secs(5)); 
        }
        });
    }


    pub async fn set_current_epoch(&mut self
    ) -> Result<(), anyhow::Error> {
        self.current_epoch=conf::set_current_epoch( self).await.unwrap();
        Ok(())
    }
    async fn timeout_err(&mut self)
    {
       
        println!("-------------------------------------------------------------------------------");
        println!("  \x1b[91m●\x1b[0m  An error occurred. It might be a network issue. ");
        println!("-------------------------------------------------------------------------------");
        let _ = self.set_current_epoch().await;
    }
    fn u64_to_ascii(mut num: u64)-> Vec<u8>
    {
        if num == 0 {
            return Vec::new();
        };
        let mut bytes: Vec<u8> = Vec::new();
        while num > 0 {
            let remainder = num % 10; // get the last digit
            num = num / 10; // remove the last digit
            bytes.push((remainder as u8) + 48);
        };
        bytes.reverse();
        return bytes
    }
}
