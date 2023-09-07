use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use log::info;

use redis_exporter::exporter::{collect_data, get_metrics_result, registry, config::CONFIG};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    init();
    info!("exporter 启动");
    registry();
    info!("注册指标完成");
    tokio::spawn(collect_data());
    info!("启动指标采集任务成功");

    HttpServer::new(|| App::new().service(get_metrics))
        .bind(("127.0.0.1", CONFIG.port))?
        .run()
        .await
}

#[get("/metrics")]
async fn get_metrics() -> impl Responder {
    let res = get_metrics_result().unwrap();
    HttpResponse::Ok().body(res)
}

fn init() {
    env_logger::init();
    info!(
        "使用的配置 redis地址:{},采集频率:{}",
        CONFIG.redis_node, CONFIG.collect_frequency
    );
}
