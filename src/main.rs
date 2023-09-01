use std::{collections::HashMap, str::FromStr};

use redis::cluster::ClusterClient;

fn main() {
    let nodes = vec![
        "redis://default:xuanwu-T3st*17@10.20.121.49:30029",
        "redis://default:xuanwu-T3st*17@10.20.121.49:30549",
        "redis://default:xuanwu-T3st*17@10.20.121.49:31089",
    ];
    let client = ClusterClient::new(nodes).unwrap();
    let mut connection = client.get_connection().unwrap();

    let info: Vec<HashMap<String, String>> = redis::cmd("INFO")
        .arg("ALL")
        .query(&mut connection)
        .unwrap();

    let res = handle_info_all(&info);
    println!("{:?}", res);
    println!("----");

    for ele in &res {
        for tuple in ele.1 {
            if tuple.0 == "uptime_in_seconds" {
                println!("{:?}", ele.0);
                println!("{:?}+{:?}", tuple.0, tuple.1)
            }
        }
    }
}

fn handle_info_all(
    info: &Vec<HashMap<String, String>>,
) -> HashMap<String, HashMap<String, String>> {
    let mut res: HashMap<String, HashMap<String, String>> = HashMap::new();
    for single_map in info {
        for single_node in single_map {
            let mut single_node_map: HashMap<String, String> = HashMap::new();
            for info_value in single_node.1.split("\r\n") {
                if !info_value.contains("#") {
                    let items: Vec<&str> = info_value.split(":").collect();
                    single_node_map.insert(
                        items.get(0).unwrap_or(&"").to_string(),
                        items.get(1).unwrap_or(&"").to_string(),
                    );
                }
            }
            res.insert(single_node.0.to_string(), single_node_map);
        }
    }
    res
}
