use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use shared_crypto::intent::Intent;
use sui_json_rpc_types::SuiTransactionBlockResponseOptions;
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use std::str::FromStr;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use crate::{conf, Miner};
use sui_sdk::types::base_types::ObjectID;
use sui_types::dynamic_field::DynamicFieldName;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{ Transaction, TransactionData};
use sui_types::{Identifier, TypeTag};
use termcolor::{ ColorChoice, StandardStream, WriteColor};
impl Miner {
    pub async fn register(&self) -> anyhow::Result<()> { 
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let miner_id = self.get_miner_id();
        let parent_object_id = ObjectID::from_hex_literal(&miner_id).expect("Failed to parse ObjectID");

        let dynamic_name = DynamicFieldName {
            type_: TypeTag::Address,
            value:  serde_json::json!(self.address.to_string()),
        };
        let dynamic_fields = self.sui_client.read_api()
            .get_dynamic_field_object(parent_object_id,dynamic_name )
            .await?;
                if let Some(data) = dynamic_fields.data {
                    let _object_id = data.object_id;

                     return Ok(());
                } else {
                    println!("New miner, Registering..........Please wait");
                    stdout.reset().unwrap();

                    let _gas_budget = 50_000_000; 
                    let gas_coins =  conf::fetch_sorted_gas_coins(&self.sui_client, &self.address).await?;
                
                    let packageid = self.get_package_id();
                    let package_id = ObjectID::from_hex_literal(&packageid).expect("Failed to parse ObjectID");
                    let mine_module = Identifier::from_str("mine").expect("err");
                    let regist_miner_function = Identifier::from_str("regist_miner").expect("err"); 
                    let mut pt_builder = ProgrammableTransactionBuilder::new();
                   
                    let objectid=pt_builder.obj(self.miner_obj).expect("Err");
                    
                    pt_builder.programmable_move_call(
                        package_id,
                        mine_module,
                        regist_miner_function,
                        vec![],
                        vec![objectid],
                    );

                    let pt = pt_builder.finish();

    

                    let gas_price = self.sui_client.read_api().get_reference_gas_price().await?;
                    let tx_data = 
                    TransactionData::new_programmable( self.address,vec![gas_coins.object_ref()],pt,50_000_000,gas_price);

                  
                    let  keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
                    let signature = keystore.sign_secure(&self.address, &tx_data, Intent::sui_transaction())?;
                         self.sui_client
                        .quorum_driver_api()
                        .execute_transaction_block(
                            Transaction::from_data(tx_data, vec![signature]),
                            SuiTransactionBlockResponseOptions::full_content(),
                            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                        )
                        .await?;

                }
                Ok(())
            }
    
}
