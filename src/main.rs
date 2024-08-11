use http::{client::ClientHttp, openai::ClientAi};
use tokio::task;

mod http;
mod toolkit;
mod config;

#[tokio::main]
async fn main() {

    // check if nmap is installed

    config::init();

    // let known_ports = config::MemData::PortData.get_str();
    // let parsed_known_ports = match serde_json::from_str::<Vec<config::PortCSV>>(&known_ports) {
    //     Ok(parsed_known_ports) => parsed_known_ports,
    //     Err(e) => {
    //         eprintln!("Error parsing known ports: {:?}", e);
    //         return;
    //     }
    // };

    // println!("parsed_known_ports: {:?}", parsed_known_ports.len());

    // let chunk_size = 2; // Smaller chunk size for more parallelism
    // let mut handles = vec![];

    // for chunk in parsed_known_ports.chunks(chunk_size) {
    //     let chunk = chunk.to_vec(); // Clone the chunk to move into the async block
    //     println!("Scanning chunk: {:?}", chunk.len());
    //     let handle = task::spawn(async move {
    //         let ports: Vec<u16> = chunk.iter().map(|port| port.port).collect(); // Assuming PortCSV has a field `port`
    //         let result = toolkit::portscan::sweeper::scan_ports("IP", &ports).await;
    //         result
    //     });
    //     handles.push(handle);
    // }

    // let mut results = vec![];

    // for handle in handles {
    //     match handle.await {
    //         Ok(result) => results.push(result),
    //         Err(e) => eprintln!("Error in port scanning task: {:?}", e),
    //     }
    // }

    // let mut reduce_result = vec![];

    // for result in results {
    //     reduce_result.extend(result);
    // }

    // println!("Reduced result: {:?}", reduce_result);


    // let result = toolkit::portscan::banner::grab_banner(
    //     "10.129.86.87",
    //     23
    // );

    // println!("result: {:?}", result);

    let res = ClientAi::new().capabilities().await;

    match res {
        Ok(res) => {
            println!("res: {:?}", res);
        }
        Err(e) => {
            println!("err: {:?}", e);
        }
    }
}

async fn scan_port(port: config::PortCSV) {
    // Your port scanning logic here
    println!("Scanning port: {:?}", port);
}