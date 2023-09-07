use std::collections::HashMap;
use std::time::Duration;

use lazy_static::lazy_static;
use log::{error, info};
use prometheus::{GaugeVec, IntGaugeVec, Opts, Registry};
use redis::cluster::ClusterClient;
use tokio::time;

use crate::{exporter::config::CONFIG, util::handle_str_to_map};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref UP_TIME_COLLECTOR: IntGaugeVec =
        IntGaugeVec::new(Opts::new("up_time", "redis_uptime"), &["instance"]).unwrap();
    pub static ref USED_MEMORY_COLLECTOR: IntGaugeVec =
        IntGaugeVec::new(Opts::new("used_memory", "redis_used_memory"), &["instance"]).unwrap();
    pub static ref CONNECTED_CLIENTS_COLLECTOR: IntGaugeVec = IntGaugeVec::new(
        Opts::new("connected_clients", "redis_connected_clients"),
        &["instance"]
    )
    .unwrap();
    pub static ref DB_KEYS_COLLECTOR: IntGaugeVec = IntGaugeVec::new(
        Opts::new("db_keys", "redis_db_keys_count"),
        &["instance", "db"]
    )
    .unwrap();
    pub static ref COMMAND_TIME_USAGE_COLLECTOR: GaugeVec = GaugeVec::new(
        Opts::new("command_usec_per_call", "command_usec_per_call"),
        &["instance", "command"]
    )
    .unwrap();
}

pub fn registry() {
    REGISTRY
        .register(Box::new(UP_TIME_COLLECTOR.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(USED_MEMORY_COLLECTOR.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(CONNECTED_CLIENTS_COLLECTOR.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(DB_KEYS_COLLECTOR.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(COMMAND_TIME_USAGE_COLLECTOR.clone()))
        .unwrap();
}

pub async fn collect_data() {
    loop {
        info!("采集");
        let nodes = vec![CONFIG.redis_node.as_str()];
        let client = ClusterClient::new(nodes).unwrap();
        let mut connection = client.get_connection().unwrap();

        let info: Vec<HashMap<String, String>> = redis::cmd("INFO")
            .arg("ALL")
            .query(&mut connection)
            .unwrap();

        let res = handle_info_all(&info);
        // println!("{:?}", res);

        for ele in &res {
            for tuple in ele.1 {
                match tuple.0.as_str() {
                    "uptime_in_seconds" => {
                        UP_TIME_COLLECTOR
                            .with_label_values(&[ele.0])
                            .set(tuple.1.parse().unwrap_or(0));
                    }
                    "used_memory" => {
                        USED_MEMORY_COLLECTOR
                            .with_label_values(&[ele.0])
                            .set(tuple.1.parse().unwrap_or(0));
                    }
                    "connected_clients" => {
                        CONNECTED_CLIENTS_COLLECTOR
                            .with_label_values(&[ele.0])
                            .set(tuple.1.parse().unwrap_or(0));
                    }
                    _ => {
                        if tuple.0.starts_with("db") {
                            let db_map = handle_str_to_map(tuple.1, ",", "=");
                            DB_KEYS_COLLECTOR.with_label_values(&[ele.0, tuple.0]).set(
                                db_map
                                    .get("keys")
                                    .unwrap_or(&String::from("0"))
                                    .parse()
                                    .unwrap(),
                            );
                        } else if tuple.0.starts_with("cmdstat_") {
                            let key_name = tuple.0.replace("cmdstat_", "");
                            let key_map = handle_str_to_map(tuple.1, ",", "=");
                            COMMAND_TIME_USAGE_COLLECTOR
                                .with_label_values(&[ele.0, key_name.as_str()])
                                .set(
                                    key_map
                                        .get("usec_per_call")
                                        .unwrap_or(&String::from("0"))
                                        .parse()
                                        .unwrap(),
                                );
                        }
                    }
                }
            }
        }
        time::sleep(Duration::from_secs(CONFIG.collect_frequency)).await;
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

pub fn get_metrics_result() -> Result<String, std::io::Error> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        error!("could not encode custom metrics: {}", e);
    };
    let mut res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            error!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        error!("could not encode prometheus metrics: {}", e);
    };
    let res_custom = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            error!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();
    res.push_str(&res_custom);
    Ok(res)
}
