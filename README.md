## mist-engine-rs      

##init
```sql
insert into mist_orders2 select * from mist_orders available_amount>0;
```

## build
```
cargo build or cargo build --release    
```          

##usage       
```
 ./target/debug/mist-engine --market_id=MT-CNYC
RUST_LOG=info ./target/debug/mist-engine-rs --market-id=MT-CNYC
```
##TODO
```
1、redis余额的更新（不用做了）
2、撮合可用余额检查（暂时不需要检查了）
3、ws远程推送增量数据（bull要改成kafka了），或者把原来的adex——engine单独推送bull
4、trade的通过锁来进行多线程消费数据
5、压力测试
6、订单取消---多开个线程专门负责取消订单
7、异步写数据库的错误处理
```

##FIXME
```
1、一些unwrap的实现更换更安全的错误处理
```



                
                 
                  