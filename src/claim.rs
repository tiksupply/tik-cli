
use std::{str::FromStr, sync::atomic::Ordering};

use shared_crypto::intent::Intent;
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use sui_json_rpc_types::SuiTransactionBlockResponseOptions;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_types::{base_types::{ObjectID, SequenceNumber}, digests::ObjectDigest, programmable_transaction_builder::ProgrammableTransactionBuilder, quorum_driver_types::ExecuteTransactionRequestType, transaction::{Transaction, TransactionData}, Identifier, TypeTag};
use async_recursion::async_recursion;

use crate::{conf, Miner};
    #[async_recursion]
    pub async fn claim(mut  miner:Miner)  {

   
       
        let mut pt_builder = ProgrammableTransactionBuilder::new();

        let claimable =  miner.rewards().await;
        let mut claimed:f64=0.0;
        if claimable>0.0
        {
           
            
            let packageid = miner.get_package_id();
          
            let package_id = ObjectID::from_hex_literal(&packageid).expect("Failed to parse ObjectID");
         
            let keystore = FileBasedKeystore::new(&sui_config_dir().unwrap().join(SUI_KEYSTORE_FILENAME)).unwrap();

         
            let treasury_o=pt_builder.obj (miner.treasury_obj).expect("Err");
            let miner_o=pt_builder.obj (miner.miner_obj).expect("Err");
            let epochs_o=pt_builder.obj(miner.epochs_obj).expect("Err");
            let clock_o=pt_builder.obj(miner.clock_obj).expect("Err");

            let claim_mod = Identifier::from_str("tikcoin").expect("err");
            let claim_function = Identifier::from_str("claim").expect("err"); 
      
            pt_builder.programmable_move_call(
                package_id,
                claim_mod,
                claim_function,
                vec![],
                vec![treasury_o,miner_o,epochs_o,clock_o],
            );
            let pt = pt_builder.finish();

            let coins = miner.sui_client
            .coin_read_api()
            .get_coins(  miner.address, None, None, None)
            .await.unwrap();
            let gascoin = coins.data;
            let mut gasbalance=0;
            let mut coins:Vec<(ObjectID, SequenceNumber, ObjectDigest)>=vec![];
            for coin in gascoin.iter() {
                gasbalance=gasbalance+coin.balance;
                coins.push(coin.object_ref());

            }
            let gas_budget = 10_000_000;
            if gasbalance<(gas_budget as u64)
            {
                eprintln!("-----------------------------------------------------------------------------------");
                eprintln!("\x1b[91m●\x1b[0m Error! Not enough balance for gas fee!  balance: {:?} SUI need:{:?} SUI", gasbalance,(gas_budget));
                eprintln!("-----------------------------------------------------------------------------------");
                return ;
            }
            let gas_price = miner.sui_client.read_api().get_reference_gas_price().await.unwrap();
            let tx_data = 
            TransactionData::new_programmable( miner.address,coins,pt,gas_budget,gas_price);
            let signature = keystore.sign_secure(&miner.address, &tx_data, Intent::sui_transaction()).unwrap();
            let trans_lock=SuiTransactionBlockResponseOptions::new();
            let transaction_response = miner.sui_client
                    .quorum_driver_api()
                    .execute_transaction_block(
                        Transaction::from_data(tx_data, vec![signature]),
                        trans_lock.with_balance_changes(),
                        Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                    )
                    .await;

            
                  
                    match transaction_response {
                        Ok(response) => {
                            response.balance_changes.iter().for_each(|items| {
                                items.iter().for_each(|item|{
                           
                                    if item.owner.get_owner_address().unwrap()==miner.address
                                    {
                                        if let Some(coin_type) = item.coin_type.clone().into(){
                                            if let TypeTag::Struct(rs) = coin_type {
                                                if let Some(name) = rs.name.into() {
                                                    if name.to_string()=="SUI"
                                                    {
                                                        println!("    Gas refund: {}   +{:?}", name,item.amount as f64/1000000000.0);
                                                    }else {
                                                        claimed=item.amount as f64/1000000000000.0;
                                                        println!("    Claimed: {}   +{:?}", name,claimed);
                                                    }
                                            
                                                }
                                            }
                                        }
                                    }
                                });
                            });
                        },
                        Err(e) => {
                            eprintln!("Error: {:?} ", e);
                          println!("-- Continue claiming  -->"); 
                          claim(miner.clone()).await;
                        }
                    }

                   

                     if claimed<claimable&&claimed>0.0 {
                        println!("-- Continue claiming  -->"); 
                          claim(miner).await;
                     }else {
                        unsafe {
                        conf::IS_CLAIMING=false;
                        }
                        conf::REWARD_IT_COUNT.store(0, Ordering::SeqCst);
                        println!("-------------------Done---------------------"); 
                     }

                    
        }else {
           
            println!("There are no claimable rewards available at the moment. Claimable: 0.");
            unsafe {
                 conf::IS_CLAIMING=false;
             }
            conf::REWARD_IT_COUNT.store(0, Ordering::SeqCst);
           
          
        }
    

    }

