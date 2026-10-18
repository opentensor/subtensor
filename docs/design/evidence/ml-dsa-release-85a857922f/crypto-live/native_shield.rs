use bittensor_core::{Client,keys::Keypair,codec::value::to_corpus_json};
use serde_json::{json,Value};
use std::io::{self,Read};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let mut input=String::new(); io::stdin().read_to_string(&mut input)?;
    let request:Value=serde_json::from_str(&input)?;
    let endpoint=request["endpoint"].as_str().ok_or("endpoint")?;
    if !endpoint.starts_with("http://127.0.0.1:") { return Err("localnet only".into()); }
    let key=Keypair::from_seed(&hex::decode(request["seed"].as_str().ok_or("seed")?)?,request["code"].as_u64().ok_or("code")? as u8)?;
    let call=hex::decode(request["call"].as_str().ok_or("call")?)?;
    let client=Client::connect(endpoint)?;
    let result=client.submit_shielded(&call,&key,true)?;
    let data:serde_json::Map<String,Value>=result.data.iter().map(|(k,v)|(k.clone(),to_corpus_json(v))).collect();
    println!("{}",json!({"success":result.success,"extrinsic_hash":result.extrinsic_hash,"block_hash":result.block_hash,"block_number":result.block_number,"extrinsic_index":result.extrinsic_index,"fee_rao":result.fee_rao,"events":result.events.iter().map(to_corpus_json).collect::<Vec<_>>(),"error":result.error.map(|e|e.to_string()),"message":result.message,"data":data}));
    Ok(())
}
