use reqwest::Client;
use reqwest::header::{HeaderMap,HeaderValue};

#[tokio::main]
async fn main(){

    let bytes = HeaderValue::from_static("bytes=1-2");
    let mut headers = HeaderMap::new();
    headers.insert("range", bytes);



    let client = Client::new()
        .get("http://downza.91speed.com.cn/2022/01/19/mysql.rar?timestamp=625774ea&auth_key=23836dfc7fb8be0b47244dc9023590ed&sign=8d48c837b4909aa9e28659236569a284&t=625782fa")
        // .headers(headers)
        .send()
        .await
        .unwrap();

    let rt = client.headers();

    for (k,v) in rt{
        println!("{:?} {:?}",k,v);
    }


}