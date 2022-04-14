use clap::Parser;
use regex::Regex;
use reqwest::Client;
use reqwest::header::{HeaderValue,HeaderMap};
use tokio::io::AsyncWriteExt;
use std::process::exit;
use anyhow::Result;

#[derive(Parser,Debug,Clone)]
#[clap(name = "godown")]
#[clap(author = "xqls")]
#[clap(version = "1.0")]
#[clap(about = "Multi-Coordination Download", long_about = "Multi-Coordination Download && super")]
pub struct Config{
    #[clap(long,short,help="download uri")]
    pub uri:String,
    #[clap(long,short,help="coordination nums")]
    pub works:usize,
}

impl Config{
    pub fn CheckArgs(&self){
        if self.works == 0 || self.works == 1 {
            eprintln!("works nums must Greater than or equal to 2");
            exit(1);
        }    

        let re =Regex::new("(https?|ftp|file)://[-A-Za-z0-9+&@#/%?=~_|!:,.;]+[-A-Za-z0-9+&@#/%=~_|]").unwrap();
        if !re.is_match(&self.uri) {
           eprintln!("download uri no match");
           exit(1);
        }
    }

    pub async fn gotfiles(&self) ->(String,usize,bool){
       let mut pass = false;
       let mut filename ="";
       let re = Regex::new(r"([^\\/]+)\.([^\\/]+)").unwrap();
       let result =re.find_iter(&self.uri).collect::<Vec<_>>();
       if result.len() > 1 {
            filename =match result.last(){
                Some(m) => {
                m.as_str()
                },
           None => {
               ""
           }
        };
       };
           

       let response =Client::new()
            .head(&self.uri)
            .send()
            .await
            .unwrap();
        let headers = response.headers();    
        match headers.get("accept-ranges"){
            Some(v) => {
                if v.eq(&HeaderValue::from_static("bytes")) {
                    pass = true 
                }
            },
            _ => pass = false,
        }
        let size =match headers.get("content-length"){
            Some(v) => {
                v.to_str().unwrap().parse().unwrap()
            },
            None => {
                0usize
            }
        };
        if size == 0{
            pass = false
        }
        if filename.eq(""){
            match headers.get("Content-Disposition"){
                Some(v) => {
                    let s = v.to_str().unwrap();
                    let re = Regex::new(r"([^\\/]+)\.([^\\/]+)").unwrap();
                    filename = match re.find(s){
                        Some(v) =>{
                            v.as_str()
                        },
                        None => {
                            ""
                        }
                    }
                },
                None =>{},
            }
        }

        if filename.eq(""){
            let mut s = String::new();
            println!("在uri和返回头部信息中都没有找到文件名称的信息，请在终端中输入并且回车");
            std::io::stdin().read_line(&mut s).unwrap();
            if s.len() == 0{
                eprintln!("输入无效,程序终止");
                exit(1);
            }
            return  (s.trim().into(),size,pass);
        }
       (filename.into(),size,pass)
    }
}

pub async fn downlaod(uri:&str,filename:&str) ->Result<()>{
    
    let response = Client::new()
        .get(uri)
        .send()
        .await?;

    let headers = response.headers();

    if let Some(v) = headers.get("content-type"){
        if v.to_str()?.contains("text") {
            eprintln!("下载终止，该链接是一个普通的uri");
            exit(1);
        }
    }

    let result = response.bytes().await?;

    tokio::fs::write(filename, result).await?;
    Ok(())

}

pub async fn block_download_run(totail:usize,config:&Config,mkdir:&str) ->Result<()>{
        let over = totail % config.works;
        let block_size = totail / config.works;
        let s =(0..config.works)
            .map(move |v|{
                let end ;
                let start = v * block_size;
                if v == (config.works - 1){
                    end = start + over + block_size; 
                }else {
                    end = (start + block_size) -1;
                }

                (v,start,end)
            })
            .collect::<Vec<_>>();
        
       tokio::fs::create_dir(mkdir).await?;
       let mut buff = vec![];
       for (i,start,end) in s {
           let t =tokio::spawn(block_download(format!("{}/{}",mkdir,i),config.uri.clone(), start, end)); 
           buff.push(t);
       }

       for t in buff {
           t.await??;
       }
        Ok(())
}

pub async fn block_download(path:String,uri:String,start:usize,end:usize) -> Result<()>{
    let stime = std::time::Instant::now();
    println!("file {:?} start",&path);
    let s = format!("bytes={}-{}",start,end).parse()?;
    let mut headers = HeaderMap::new();
    headers.insert("range",s); 

    let response = Client::new()
        .get(uri)
        .headers(headers)
        .send()
        .await?;
    let block = response.bytes().await?;

    tokio::fs::write(&path, block).await?;

    let etime = stime.elapsed();

    println!("file {:?} use {:?}",&path,etime);

    Ok(())
}

pub async fn  merge_file(savepath:&str,block_file:&str,nums:usize) ->Result<()>{

    let mut file =tokio::fs::File::create(savepath).await?;
    for i in 0..nums{
        let mut content = tokio::fs::File::open(&format!("{}/{}",block_file,i)).await?;
        tokio::io::copy(&mut content,&mut file).await?;
        content.shutdown().await?;
    }
    file.shutdown().await?;

    tokio::fs::remove_dir_all(block_file).await?;
    Ok(())
}

