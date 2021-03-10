use crate::package::{ContractInfoResponse, OfferingsResponse, QueryOfferingsResult};
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, HandleResponse, HumanAddr,
    InitResponse, MessageInfo, Order, StdResult, WasmMsg
};

use cosmwasm_std::KV;

use std::str::from_utf8;

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, QueryMsg, SellNft};
use crate::state::{increment_offerings, Offering, CONTRACT_INFO, OFFERINGS};
use cw721::{Cw721HandleMsg, Cw721ReceiveMsg};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<InitResponse, ContractError> {
    let info = ContractInfoResponse { name: msg.name };
    CONTRACT_INFO.save(deps.storage, &info)?;
    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::BuyNft {
            spender,
            amount,
            offering_id,
        } => execute_buy_nft(deps, info, spender, amount, offering_id),
        HandleMsg::ReceiveNft(msg) => try_receive_nft(deps, info, msg),
    }
}

pub fn execute_buy_nft(
    deps: DepsMut,
    info: MessageInfo,
    spender: HumanAddr,
    amount: Coin,
    offering_id: String,
) -> Result<HandleResponse, ContractError> {

    let offering = OFFERINGS.load(deps.storage, &offering_id)?;

    if amount.amount < offering.list_price.amount {
        return Err(ContractError::InsufficientFunds {});
    }

    BankMsg::Send {
        from_address: spender.clone(),
        to_address: deps.api.human_address(&offering.seller)?,
        amount: vec![amount],
    };

    let transfer_cw721_msg = Cw721HandleMsg::TransferNft {
        recipient: spender.clone(),
        token_id: offering.token_id.clone(),
    };
    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: deps.api.human_address(&offering.contract_addr)?,
        msg: to_binary(&transfer_cw721_msg)?,
        send: vec![],
    };

    // transfer nft to buyer
    let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();

    // let cosmos_msgs = vec![cw20_transfer_cosmos_msg, cw721_transfer_cosmos_msg];
    let cosmos_msgs = vec![cw721_transfer_cosmos_msg];

    OFFERINGS.remove(deps.storage, &offering_id);

    // let price_string = format!("{} {}", amount, info.sender);

    Ok(HandleResponse {
        messages: cosmos_msgs,
        attributes: vec![
            attr("action", "buy_nft"),
            attr("buyer", spender),
            attr("seller", offering.seller),
            // attr("paid_price", price_string),
            attr("token_id", offering.token_id),
            attr("contract_addr", offering.contract_addr),
        ],
        data: None,
    })
}

pub fn try_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    let msg: SellNft = match rcv_msg.msg {
        Some(bin) => Ok(from_binary(&bin)?),
        None => Err(ContractError::NoData {}),
    }?;

    let id = increment_offerings(deps.storage)?.to_string();

    let off = Offering {
        contract_addr: deps.api.canonical_address(&info.sender)?,
        token_id: rcv_msg.token_id,
        seller: deps.api.canonical_address(&rcv_msg.sender)?,
        list_price: msg.list_price.clone(),
    };

    OFFERINGS.save(deps.storage, &id, &off)?;

    let price_string = format!("{} {}", msg.list_price.amount, msg.list_price.denom);

    Ok(HandleResponse {
        messages: Vec::new(),
        attributes: vec![
            attr("action", "sell_nft"),
            attr("original_contract", info.sender),
            attr("seller", off.seller),
            attr("list_price", price_string),
            attr("token_id", off.token_id),
        ],
        data: None,
    })
}

pub fn query(deps: DepsMut, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOfferings {} => to_binary(&query_offerings(deps)?),
    }
}

fn query_offerings(deps: DepsMut) -> StdResult<OfferingsResponse> {
    let res: StdResult<Vec<QueryOfferingsResult>> = OFFERINGS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_offering(deps.api, kv_item))
        .collect();

    Ok(OfferingsResponse {
        offerings: res?, // Placeholder
    })
}

fn parse_offering(api: &dyn Api, item: StdResult<KV<Offering>>) -> StdResult<QueryOfferingsResult> {
    item.and_then(|(k, offering)| {
        let id = from_utf8(&k)?;
        Ok(QueryOfferingsResult {
            id: id.to_string(),
            token_id: offering.token_id,
            list_price: offering.list_price,
            contract_addr: api.human_address(&offering.contract_addr)?,
            seller: api.human_address(&offering.seller)?,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, HumanAddr, Uint128};

    // #[test]
    // fn proper_initialization() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(17, value.count);
    // }

    // #[test]
    // fn execute_buy_nft() {
    // }

    #[test]
    fn sell_offering_happy_path() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg {
            name: String::from("test market"),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));

        let sell_msg = SellNft {
            // address: HumanAddr::from("cw20ContractAddr"),
            list_price: Coin {
                denom: "ATOM".to_string(),
                amount: Uint128(5),
            },
        };

        let msg = HandleMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: HumanAddr::from("seller"),
            token_id: String::from("SellableNFT"),
            msg: to_binary(&sell_msg).ok(),
        });
        let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Offering should be listed
        let res = query(deps.as_mut(), mock_env(), QueryMsg::GetOfferings {}).unwrap();
        let value: OfferingsResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.offerings.len());

        let buy_msg = HandleMsg::BuyNft {
            spender: HumanAddr::from("buyer"),
            amount: Coin {
                denom: "ATOM".to_string(),
                amount: Uint128(5),
            },
            offering_id: value.offerings[0].id.clone(),
        };

        // let info_buy = mock_info("cw20ContractAddr", &coins(2, "token"));
        // let _res = handle(&mut deps, mock_env(), info_buy, msg2).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let _res = handle(deps.as_mut(), mock_env(), info, buy_msg).unwrap();

        // check offerings again. Should be 0
        let res2 = query(deps.as_mut(), mock_env(), QueryMsg::GetOfferings {}).unwrap();
        let value2: OfferingsResponse = from_binary(&res2).unwrap();
        assert_eq!(0, value2.offerings.len());
    }
}
