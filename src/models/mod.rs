use postgres::{config::Config, error::Error, row::SimpleQueryRow, Client, NoTls};

extern crate rustc_serialize;

use rustc_serialize::json;

use serde::Deserialize;
use std::ops::Mul;
/*
 id               | text                        |          | not null |
 trader_address   | text                        |          |          |
 market_id        | text                        |          |          |
 side             | text                        |          |          |
 price            | numeric(32,8)               |          |          |
 amount           | numeric(32,8)               |          |          |
 status           | text                        |          |          |
 type             | text                        |          |          |
 available_amount | numeric(32,8)               |          |          |
 confirmed_amount | numeric(32,8)               |          |          |
 canceled_amount  | numeric(32,8)               |          |          |
 pending_amount   | numeric(32,8)               |          |          |
 updated_at       | timestamp without time zone |          |          |
 created_at       | timestamp without time zone |          |          |
 signature        | text                        |          |          |
 expire_at        | bigint                      |          |          |
*/

#[derive(Deserialize, Debug, Default)]
pub struct UpdateOrder {
    pub id: String,
    pub trader_address: String,
    pub status: String,
    pub amount: f64,
    pub available_amount: f64,
    pub confirmed_amount: f64,
    pub canceled_amount: f64,
    pub pending_amount: f64,
    pub updated_at: String,
}

#[derive(Deserialize, RustcDecodable, RustcEncodable, Debug, Default, Clone)]
pub struct EngineOrder {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: String,
    created_at: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct TradeInfo {
    id: i32,
    transaction_id: i32,
    transaction_hash: String,
    status: String,
    market_id: String,
    maker: String,
    taker: String,
    price: String,
    amount: String,
    taker_side: String,
    maker_order_id: String,
    taker_order_id: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct MarketVolume {
    pub marketID: String,
    pub volume: f64,
}

use std::sync::Mutex;
use std::ptr::null;


pub fn get_change_rate(marketID: &str) -> f64 {
    let current_price = get_current_price_marketID(marketID);
    if current_price == 0.0 { return 0.0; }

    let mut yesterday_price = 0.0;
    let sql = "select cast(price as float8) from mist_trades_tmp where (current_timestamp - created_at) < '24 hours' and market_id=$1 order by created_at  limit 1";
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[&marketID]);
    if let Err(err) = result {
        println!("get_change_rate failed {:?}", err);
        if !crate::restartDB() {
            return 0.0;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[&marketID]);
    }
    let rows = result.unwrap();
    for row in rows {
        yesterday_price = row.get(0);
    }
    if yesterday_price == 0.0 { return 0.0; }
    let rate = (current_price - yesterday_price) / yesterday_price;
    rate.mul(100000000.0).floor() / 100000000.0
}


pub fn get_current_price_marketID(id: &str) -> f64 {
    let sql = "select cast(price as float8) from mist_trades_tmp where (current_timestamp - created_at) < '24 hours' and market_id=$1 order by created_at desc limit 1";
    let mut price: f64 = 0.0;
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[&id]);

    if let Err(err) = result {
        println!("get_marketID_volume failed {:?}", err);
        if !crate::restartDB() {
            return 0.0;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[&id]);
    }
    let rows = result.unwrap();
    for row in rows {
        price = row.get(0);
    }
    price
}
/*
pub fn insert_trade(trade: &TradeInfo){
    let sql = "select market_id,cast(sum(amount) as float8) as volume  from mist_trades_tmp  where (current_timestamp - created_at) < '24 hours' group by market_id";
    let mut markets: Vec<MarketVolume> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    if let Err(err) = result {
        println!("get_marketID_volume failed {:?}", err);
        if !crate::restartDB() {
            return;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    }
    let rows = result.unwrap();
    for row in rows {
        let info = MarketVolume {
            marketID: row.get(0),
            volume: row.get(1),
        };
        markets.push(info);
    }
}
*/
pub fn update_order(order: &UpdateOrder) {
    // fixme:注入的写法暂时有问题，先直接拼接
    let sql =
        format!("UPDATE mist_orders2 SET (available_amount,confirmed_amount,canceled_amount,pending_amount,status,updated_at)=\
                ({},confirmed_amount,canceled_amount,{},'{}','{}') WHERE id='{}'",
                order.available_amount, order.pending_amount, order.status, order.updated_at, order.id);
    println!("--{}-",sql);
    let mut result = crate::CLIENTDB.lock().unwrap().execute(&*sql, &[]);
    if let Err(err) = result {
        println!("update_order failed {:?} {}", err,sql);
        if !crate::restartDB() {
            return;
        }
        result = crate::CLIENTDB.lock().unwrap().execute(&*sql, &[]);
    }
    println!("success update {} rows", result.unwrap());
    return;
}

pub fn get_order(id: &str) -> UpdateOrder {
    let sql = "select id,trader_address,status,\
             cast(amount as float8),\
            cast(available_amount as float8),\
            cast(confirmed_amount as float8),\
            cast(canceled_amount as float8),\
            cast(pending_amount as float8),\
            cast(updated_at as text) \
            from mist_orders2 where id=$1";
    let mut order: UpdateOrder = Default::default();
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[&id]);
    if let Err(err) = result {
        println!("get UpdateOrder failed {:?}", err);
        if !crate::restartDB() {
            return order;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[&id]);
    }
    //id 唯一，直接去第一个成员
    let rows = result.unwrap();
    order = UpdateOrder {
        id: rows[0].get(0),
        trader_address: rows[0].get(1),
        status: rows[0].get(2),
        amount: rows[0].get(3),
        available_amount: rows[0].get(4),
        confirmed_amount: rows[0].get(5),
        canceled_amount: rows[0].get(6),
        pending_amount: rows[0].get(7),
        updated_at: rows[0].get(8),
    };
    order
}

pub fn list_available_orders(side: &str, channel: &str) -> Vec<EngineOrder> {
    let mut sort_by = "ASC";
    if side == "buy" {
        sort_by = "DESC";
    }
    let sql = format!("select id,cast(price as float8),cast(available_amount as float8),side,cast(created_at as text) from mist_orders2 \
    where market_id='{}' and available_amount>0 and side='{}' order by price {} ,created_at ASC limit 10", channel, side, sort_by);
    println!("list_available_orders failed333 {}", sql);
    let mut orders: Vec<EngineOrder> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    if let Err(err) = result {
        println!("get_active_address_num failed {:?}", err);
        if !crate::restartDB() {
            return orders;
        }
        result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    }
    let rows = result.unwrap();
    for row in rows {
        let info = EngineOrder {
            id: row.get(0),
            price: row.get(1),
            amount: row.get(2),
            side: row.get(3),
            created_at: row.get(4),
        };
        orders.push(info);
    }
    println!("list_available_orders 444 {:?}------44", orders);
    orders
}
