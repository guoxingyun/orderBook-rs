use super::engine::EngineTrade;
use crate::models::*;
use crate::util::*;
use std::cmp::Ord;
use std::collections::BTreeMap;
use std::time::SystemTime;

use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
use kafka::error::Error as KafkaError;
use serde::Deserialize;
use std::any::Any;
use std::env;
use std::ops::Mul;

//  "pending","partial_filled","cancled","full_filled" or ""
pub fn update_maker(order: &mut UpdateOrder, engine_trade: &EngineTrade) -> bool {
    // todo:更新redis余额
    order.available_amount = (order.available_amount - engine_trade.amount).to_fix(4);
    order.pending_amount = (order.pending_amount + engine_trade.amount).to_fix(4);
    order.updated_at = get_current_time();
    if order.available_amount > 0.0 && order.available_amount < order.amount {
        order.status = "partial_filled".to_string();
    } else if order.available_amount == 0.0 {
        order.status = "full_filled".to_string();
    } else {
        info!("Other circumstances that were not considered, or should not have occurred");
    }
    crate::models::update_order(order);
    true
}

pub fn insert_taker(taker_order: &mut OrderInfo, engine_trade: &EngineTrade) -> bool {
    // todo:更新redis余额
    taker_order.available_amount = (taker_order.available_amount - engine_trade.amount).to_fix(4);
    taker_order.pending_amount = (taker_order.pending_amount + engine_trade.amount).to_fix(4);
    taker_order.updated_at = get_current_time();
    if taker_order.available_amount > 0.0 && taker_order.available_amount < taker_order.amount {
        taker_order.status = "partial_filled".to_string();
    } else if taker_order.available_amount == 0.0 {
        taker_order.status = "full_filled".to_string();
    } else {
        error!("Other circumstances that were not considered, or should not have occurred");
    }
    let order_info = struct2array(taker_order);
    crate::models::insert_order2(order_info);
    true
}

pub fn generate_trade(
    taker: &str,
    maker_order: &UpdateOrder,
    engine_trade: &EngineTrade,
    transaction_id: i32,
) -> Vec<String> {
    // todo:更新redis余额
    //fixme::默认值设计
    unsafe {
        let mut trade = TradeInfo {
            id: "".to_string(),
            transaction_id,
            transaction_hash: "".to_string(),
            status: "matched".to_string(),
            market_id: crate::market_id.clone(),
            maker: maker_order.trader_address.clone(),
            taker: taker.to_string(),
            price: engine_trade.price,
            amount: engine_trade.amount,
            taker_side: engine_trade.taker_side.clone(),
            maker_order_id: engine_trade.maker_order_id.clone(),
            taker_order_id: engine_trade.taker_order_id.clone(),
            updated_at: get_current_time(),
            created_at: get_current_time(),
        };
        let data = format!(
            "{}{}{}{}{}{}{}{}{}",
            trade.market_id,
            trade.maker,
            trade.taker,
            trade.price,
            trade.amount,
            trade.taker_side,
            trade.maker_order_id,
            trade.taker_order_id,
            trade.created_at
        );
        let txid = sha256(data);
        trade.id = txid;
        let trade_arr = struct2array(&trade);
        trade_arr
    }
}
