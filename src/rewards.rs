use sui_json_rpc_types::{SuiMoveStruct, SuiMoveValue, SuiObjectDataOptions, SuiParsedData};
use sui_types::{base_types::ObjectID, dynamic_field::DynamicFieldName , TypeTag};

use crate::{conf, Miner};
use std::collections::HashMap;
struct Rewardata  {
    id: ObjectID,
    share: u64,
    unlock:u64,
    eid:u64, 
}

impl Miner {
   
    pub async fn rewards(&mut self) -> f64{
        let mut claimable:f64=0.0;
        let mut locked:f64=0.0;
        let mut epdatamap:HashMap<ObjectID, u64> = HashMap::new();
        let parent_object_id = ObjectID::from_hex_literal(self.get_miner_id()).expect("Failed to parse ObjectID");
        let now=conf::set_current_epoch( self).await.unwrap();
        let dynamic_name = DynamicFieldName {
            type_: TypeTag::Address,
            value:  serde_json::json!(self.address.to_string()),
        };
           
        let miner_obj_fields = self.sui_client.read_api()
            .get_dynamic_field_object(parent_object_id,dynamic_name )
            .await.expect("Get data err.");

            if let Some(sui_object_data) = miner_obj_fields.data {

                let mut next_curs=None;
                let mut has_next=true;

                while has_next {
                    let mut reward_eps: Vec<ObjectID>=vec![];
                    let mut reward_oids:Vec<ObjectID>=vec![];
                    let mut reward_data:Vec<Rewardata>=vec![];
                    let miner_data_fields=self.sui_client.read_api().get_dynamic_fields(sui_object_data.object_id, next_curs, None).await;
                    let miner_data_fields_clone=miner_data_fields.unwrap().clone();
                    has_next=miner_data_fields_clone.has_next_page;
                    next_curs=miner_data_fields_clone.next_cursor;

                    let info_item=miner_data_fields_clone.data;

                    info_item.iter().for_each(|item| {
                        reward_eps.push(item.object_id);
                    });
                
                    let si_object_data_options=SuiObjectDataOptions::new(
                       
                    );
          
                    let rewarddatas=self.sui_client.read_api().multi_get_object_with_options(reward_eps, 
                        
                        si_object_data_options.clone().with_content()

                   ).await;

                    rewarddatas.unwrap().iter().for_each(|item| {
                        let item_clone=item.clone();
                      
                        if let Some( SuiParsedData::MoveObject(move_object)) =  item_clone.data.unwrap().content {
                         if let SuiMoveStruct::WithFields(fields) = move_object.fields {
                            let mut redata=Rewardata{ id: ObjectID::random(), share: 0, unlock:0,eid:0};
                             if let Some(idd) = fields.get("euid") {
                                match idd {
                                    SuiMoveValue::Vector(idds) => {
                                        let mut bytes = Vec::new();
                                        for number in  idds {
                                            match number {
                                                SuiMoveValue::Number(js) => {
                                                    bytes.push(*js as u8);
                                                         
                                                   },
                                                   _ => {
                                                
                                                   }
                         
                                            }
                                        }
                                        redata.id=ObjectID::from_bytes(bytes).unwrap();
                                        reward_oids.push(redata.id)

                                    },
                                    _ => {
                                 
                                    }
                                }
                             }
                             let mut _share: u64=0;
                             if let Some(shares) = fields.get("share") {
                                match shares {
                                    SuiMoveValue::String(js) => {
                                     _share=js.parse().unwrap();
                                     redata.share=_share;
                                          
                                    },
                                    _ => {
                                 
                                    }
                                }
                             }
                             let mut _unlock: u64=0;
                             if let Some(unl) = fields.get("unlock") {
                                match unl {
                                    SuiMoveValue::String(js) => {
                                    _unlock=js.parse().unwrap();
                                    redata.unlock=_unlock;
                                          
                                    },
                                    _ => {
                                 
                                    }
                                }
                             }
                             let mut _eid: u64=0;
                             if let Some(ep) = fields.get("eid") {
                                match ep {
                                    SuiMoveValue::String(js) => {
                                    _eid=js.parse().unwrap();
                                    redata.eid=_eid;
                                          
                                    },
                                    _ => {
                                 
                                    }
                                }
                             }
                             reward_data.push(redata)

                         }
                        }
                     });


                     let epdatas=self.sui_client.read_api().multi_get_object_with_options(reward_oids, 
                        
                        si_object_data_options.with_content()

                   ).await;

                   epdatas.unwrap().iter().for_each(|item| { 
                    let item_clone=item.clone();
                    if let Some( SuiParsedData::MoveObject(move_object)) =  item_clone.data.unwrap().content {
                        if let SuiMoveStruct::WithFields(fields) = move_object.fields {
                            if let Some(sha_min) = fields.get("shares_miners") {
                              
                                match sha_min {
                                    SuiMoveValue::Vector(sha_min_vec) => {
                                        let all_share:u64=sha_min_vec[0].to_string().parse().unwrap();
                                        epdatamap.insert(item.object_id().unwrap(), all_share);
                                    }
                                    ,
                                    _ => {
                                 
                                    }
                           
                                }
                            }
                        }
                    }

                   });

                 reward_data.iter().for_each(|item| { 
                    let allshare=(*epdatamap.get(&item.id).unwrap()) as f64;
                    let rate= (item.share as f64)/  allshare;
                    let rwd=rate*conf::EPOCH_REWARD;

                    

                    if item.unlock<=now-30
                    {
                        claimable=claimable+rwd;
                        println!("      Epoch: {:?}    Share: {:?}  Total share: {:?}   Reward: {}  ",item.eid,item.share as u64,allshare as u64,  Self::format_float(rwd * conf::MINER_SHARE, 12)  );
                    }else  {
                        locked=locked+rwd;
                        let seconds = if let Some(value) = (item.unlock).checked_sub(30+now) {
                            value
                        } else {
                            30
                        };

                      
                        let days = seconds / (24 * 3600);
                        let hours = (seconds % (24 * 3600)) / 3600;
                        let minutes = (seconds % 3600) / 60;
                        let seconds = seconds % 60;
                        println!("      Epoch: {:?}    Share: {:?}  Total share: {:?}   Reward: {}  Unlock: {}D {}H {}M {}S ",item.eid,item.share as u64,allshare as u64,   Self::format_float(rwd* conf::MINER_SHARE,12),days,hours, minutes,seconds);
                    }
                 });

                      
                }

             

                println!("-------------------------------------------------");
                eprintln!("      Reward: {:?}    TIK  ", Self::truncate_float((claimable+locked)* conf::MINER_SHARE, 12));
                eprintln!("      Claimable: {:?}  TIK  ", Self::truncate_float(claimable* conf::MINER_SHARE, 12));
                eprintln!("      Locked: {:?}    TIK  ", Self::truncate_float(locked* conf::MINER_SHARE, 12));
                println!("-------------------------------------------------");
     

             
              
            }
            return claimable;
          

    }

    fn truncate_float(num: f64, precision: usize) -> f64 {
        let multiplier = 10_f64.powi(precision as i32);
        (num * multiplier).trunc() / multiplier
    }
    fn format_float(num: f64, decimal_places: usize) -> String {
        format!("{:.1$}", num, decimal_places)
    }
  
}
