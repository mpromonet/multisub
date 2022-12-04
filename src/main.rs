extern crate redis;
use redis::Commands;
use std::{env, thread, time::Duration};

extern crate hostname;

fn connect(url: &str) -> redis::Connection {
    redis::Client::open(url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

fn subscribetokey(mut readcon: redis::Connection, mut subcon: redis::Connection, key: &str) -> ! {
    let mut pubsub = subcon.as_pubsub();
    let _ = pubsub.psubscribe(format!("__key*__:{}", key));
    let _ = pubsub.set_read_timeout(Some(Duration::from_millis(5000)));

    loop {

        let res = pubsub.get_message();

        // renew
        let _: i32 = readcon.pexpire("leader", 10000).unwrap();

        match res {
            Ok(msg) => {
                let payload: String = msg.get_payload().unwrap();
                println!("channel '{}': {}", msg.get_channel_name(), payload);
                // get key name
                let strings: Vec<&str> = msg.get_channel_name().split(":").collect();
                let key = strings[strings.len()-1];
        
                let res: Result<i32, redis::RedisError> = readcon.get(key);
                // print value
                match res {
                    Ok(count) => println!("{} = {}", key, count),
                    Err(error) => println!("{} = {}", key, error.category()),
                }
                // print ttl
                let ttl: i32 = readcon.ttl(key).unwrap();
                println!("{} ({})", key, ttl);
            },
            Err(error) => println!("{} = {}", key, error.category()),
        }
    }
}

fn elect(mut con: redis::Connection) -> Result<redis::Value, redis::RedisError> {
    let hostname = hostname::get().unwrap().into_string().unwrap();
    println!("Hostname: {:?}", hostname);
     redis::cmd("SET")
    .arg("leader")
    .arg(hostname)
    .arg("PX")
    .arg(10000)
    .arg("NX").query(&mut con)
}

fn main() {
    let mut url = "redis://127.0.0.1/";
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        url = &args[1];
    }

    // write something
    let mut con = connect(url);
    let _: () = con.set("my_key", 42).unwrap();
    let _: () = con.expire("my_key", 10).unwrap();

    let count: i32 = con.get("my_key").unwrap();
    println!("my_key = {}", count);

 
    loop {
        let mut con = connect(url);
        let _ : () = redis::cmd("CONFIG")
        .arg("SET")
        .arg("notify-keyspace-events")
        .arg("KEA")
        .query(&mut con)
        .unwrap();

        let res = elect(con);
        match res {
            Ok(redis::Value::Okay) => {
                println!("leader");
                let readcon = connect(url);
                let subcon = connect(url);
                subscribetokey(readcon, subcon, "my_key");
            },
            _ => {
                println!("leader failed");
                thread::sleep(Duration::from_millis(5000));
            }
        }    
    }

}
