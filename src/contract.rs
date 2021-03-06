use crate::package::{OfferingsResponse, QueryOfferingsResult};
use cosmwasm_std::{
    to_binary, Api, Binary, Coin, Deps, DepsMut, Env, HandleResponse, InitResponse, MessageInfo,
    Order, Querier, StdResult, Storage,
};
// use cw_storage_plus::{Order};

use crate::error::ContractError;
use crate::msg::{BuyNft, CountResponse, HandleMsg, InitMsg, QueryMsg, SellNft};
use crate::state::{config, config_read, Offering, OFFERINGS};
use cw721::{Cw721HandleMsg, Cw721ReceiveMsg};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<InitResponse, ContractError> {
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
        HandleMsg::Receive(msg) => try_receive(deps, info, msg),
        HandleMsg::ReceiveNft(msg) => try_receive_nft(deps, info, msg),
    }
}

pub fn try_receive(
    deps: DepsMut,
    info: MessageInfo,
    rcv_msg: Coin,
) -> Result<HandleResponse, ContractError> {
    Ok(HandleResponse::default())
}

pub fn try_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    Ok(HandleResponse::default())
}

pub fn query(deps: DepsMut, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOfferings {} => to_binary(&query_offerings(deps)?),
    }
}

fn query_offerings(deps: DepsMut) -> StdResult<OfferingsResponse> {
    let res: StdResult<Vec<QueryOfferingsResult>> = OFFERINGS
        .range(&deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_offering(deps.api, kv_item))
        .collect();

    Ok(OfferingsResponse {
        offerings: res?, // Placeholder
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
    // fn increment() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Increment {};
    //     let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let res = handle(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let _res = handle(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
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

        // let buy_msg = BuyNft {
        //     offering_id: value.offerings[0].id.clone(),
        // };

        // let msg2 = HandleMsg::Receive(Cw20ReceiveMsg {
        //     sender: HumanAddr::from("buyer"),
        //     amount: Uint128(5),
        //     msg: to_binary(&buy_msg).ok(),
        // });

        // let info_buy = mock_info("cw20ContractAddr", &coins(2, "token"));

        // let _res = handle(&mut deps, mock_env(), info_buy, msg2).unwrap();

        // // check offerings again. Should be 0
        // let res2 = query(&deps, mock_env(), QueryMsg::GetOfferings {}).unwrap();
        // let value2: OfferingsResponse = from_binary(&res2).unwrap();
        // assert_eq!(0, value2.offerings.len());
    }
}
