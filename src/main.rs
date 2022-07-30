extern crate redis;
use redis::Commands;

fn connect(url: &str) -> redis::Connection {
    redis::Client::open(url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

fn main() {
    let url = "redis://127.0.0.1/";
    let mut con = connect(url);
    let _: () = con.set("my_key", 42)
                    .expect("SET failed");
    let _: () = con.incr("my_key", 2)
                    .expect("INCR failed");
    let _: () = con.expire("my_key", 10)
                    .expect("EXPIRE failed");

    let count: i32 = con.get("my_key")
                         .expect("GET failed");
    println!("my_key = {}", count);

    let _ : () = redis::cmd("CONFIG")
                    .arg("SET")
                    .arg("notify-keyspace-events")
                    .arg("KEA")
                    .query(&mut con)
                    .expect("CONFIG failed");

    let mut subcon = connect(url);
    let mut pubsub = subcon.as_pubsub();
    let _ = pubsub.psubscribe("__key*__:my_key");

    loop {
        let msg = pubsub.get_message().expect("GET MESSAGE failed");
        let payload : String = msg.get_payload().expect("GET PAYLOAD failed");
        println!("channel '{}': {}", msg.get_channel_name(), payload);
        let strings: Vec<&str> = msg.get_channel_name().split(":").collect();
        let key = strings[strings.len()-1];
        let count: i32 = con.ttl(key)
            .expect("ttl failed");
        println!("ttl {} = {}", key, count);
    }
}
