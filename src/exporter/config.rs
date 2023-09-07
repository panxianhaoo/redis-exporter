use std::process::exit;
use std::{env, fs};

use clap::{command, Parser};
use lazy_static::lazy_static;
use log::{error, info};
use serde_derive::Deserialize;

lazy_static! {
    pub static ref CONFIG: ExporterConfig = ExporterConfig::parse();
}

#[derive(Deserialize, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct ExporterConfig {
    /// 暴露的监控端口
    #[arg(short, long, default_value_t = 8090)]
    pub port: u16,

    /// 采集频率
    #[arg(short, long, default_value_t = 10)]
    pub collect_frequency: u64,

    /// redis地址
    #[arg(short, long)]
    pub redis_node: String,
}

#[allow(dead_code)]
pub fn read_config_file() -> ExporterConfig {
    let mut filename = "exporter_config.toml".to_string();
    let custom_config_file = env::var("REDIS_EXPORTER_CONFIG");
    if custom_config_file.is_ok() {
        filename = custom_config_file.unwrap();
    }
    let contents = match fs::read_to_string(&filename) {
        Ok(c) => c,
        Err(_) => {
            error!("无法读取配置文件:{:?}", filename);
            exit(1);
        }
    };
    let config: ExporterConfig = match toml::from_str(&contents) {
        Ok(c) => c,
        Err(e) => {
            error!("解析配置文件错误,{:?}", e);
            exit(1);
        }
    };
    info!("读取配置文件完成");
    config
}
