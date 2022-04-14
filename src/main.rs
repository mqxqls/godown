use std::ops::Deref;

use clap::Parser;
use godown::{Config, downlaod, block_download_run, merge_file};
use once_cell::sync::Lazy;
use std::time::Instant;

static CONFIG:Lazy<Config> = Lazy::new(||{Config::parse()}); 

#[tokio::main]
async fn main() {
    
    //检查uri,协程数量,参数
    CONFIG.CheckArgs();

    let startTime = Instant::now();

    //获取下载的文件名称和容量
    let (filename,totail,one) = CONFIG.gotfiles().await;
    let mkdir = format!("{}{}",filename,CONFIG.works);

    if !one || totail < 10485760 {
        downlaod(&CONFIG.uri, &filename).await.map_err(|e|{eprintln!("{}",e.to_string())});
    }else {
        block_download_run(totail, &CONFIG, &mkdir).await.map_err(|e|{eprintln!("{}",e.to_string())});
        merge_file(&filename, &mkdir, CONFIG.works).await.map_err(|e|{eprintln!("{}",e.to_string())});
    }
   
    let end =startTime.elapsed();
    println!("use time for {:?}",end);
}
