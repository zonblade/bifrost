use http::client::ClientAi;

mod http;
mod toolkit;
mod config;

#[tokio::main]
async fn main() {

    // check if nmap is installed

    config::init();

    toolkit::portscan::sweeper::scan_ports(
        "103.82.92.19",
        &[80, 443, 6379, 8899, 22, 21, 2222, 8085, 8086, 8087],
    );

    let result = toolkit::portscan::banner::grab_banner(
        "103.82.92.19",
        22
    );

    println!("result: {:?}", result);

    let res = ClientAi::new().invoke().await;

    match res {
        Ok(res) => {
            println!("res: {:?}", res);
        }
        Err(e) => {
            println!("err: {:?}", e);
        }
    }
}
