use cosmwasm_std::{Coin, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractInfoResponse {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct QueryOfferingsResult {
    pub id: String,
    pub token_id: String,
    pub list_price: Coin,
    pub contract_addr: HumanAddr,
    pub seller: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OfferingsResponse {
    pub offerings: Vec<QueryOfferingsResult>,
}
