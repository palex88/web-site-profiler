use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;
use std::collections::HashMap;

#[macro_use]
extern crate clap;
use clap::App;

struct HttpReturn {
    /// Struct to hold specific parts of the HTTP Response
    headers: Vec<String>,
    body: String,
    total_time: u128,
    response_size: usize,
}

/// Parse the cli args and sends out for processing.
/// 
/// CLI Args:
/// 
///     --url       -u: The full url to be tested
///     --profile   -p: The number of times to run the test
///     --help      -h: Show CLI help tool
fn main() {

    let site: String;
    let num_runs: i32;

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    if let Some(url) = matches.value_of("url") {
        site = String::from(url);
        let (url, path) = parsr_url(site);
        if let Some(profile) = matches.value_of("profile") {
            num_runs = String::from(profile).parse().unwrap();
        } else {
            num_runs = 1;
        }

        if num_runs > 1 {
            run(url.clone(), path.clone(), num_runs);
        } else {
            let res = connect(url.clone(), path.clone());
            match res {
                Ok(v) => {
                    println!("Headers       :");
                    for header in v.headers {
                        println!("\t{}", header);
                    }
                    println!("Body          :");
                    println!("{}", v.body);
                    println!("Total Time    : {}", v.total_time);
                    println!("Response Size : {}", v.response_size);
                },
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}

/// Runs the program a given num of times
/// Does not contain return, instead prints our summary of all runs
/// 
/// Args:
/// 
///     url: the root URL that should be tested
///     path: additional URL path information
///     num_runs: the number of times to run the tool
fn run(url: String, path: String, num_runs: i32) {
    
    let mut times: u128 = 0;
    let mut fastest: u128 = u128::MAX;
    let mut slowest: u128 = u128::MIN;
    let mut median: Vec<u128> = Vec::new();
    let mut response_codes: HashMap<String, i32> = HashMap::new();
    let mut smallest: usize = usize::MAX;
    let mut largest: usize = usize::MIN;

    for _ in 0..num_runs {
        let res = connect(url.clone(), path.clone());
        match res {
            Ok(v) => {
                let time = v.total_time;
                times += time;
                median.push(time);
                if time > slowest {
                    slowest = v.total_time;
                }
                if time < fastest {
                    fastest = v.total_time;
                }

                let size = v.response_size;
                if size > largest {
                    largest = size;
                }
                if size < smallest {
                    smallest = size;
                }

                let code_line: Vec<&str> = v.headers[0].split_whitespace().collect();
                let code = String::from(code_line[1]);
 
                let counter = response_codes.entry(code).or_insert(0);
                *counter += 1;
            },
            Err(e) => println!("Error: {}", e),
        }   
    }

    // Prints out the results of all the runs, along with some general statistics.
    median.sort();
    println!("Fastest Time (NS)      : {}", fastest);
    println!("Slowest Time (NS)      : {}", slowest);
    println!("Mean Time (NS)         : {}", (times/median.len() as u128));
    println!("Median Time (NS)       : {}", median[median.len()/2]);
    println!("Smallest Response Size : {}", smallest);
    println!("Largest Response Size  : {}", largest);
    if response_codes.contains_key("200") {
        println!("Successfull Attempts   : {}/{}", response_codes["200"], median.len());
    } else {
        println!("Successfull Attempts   : 0/{}", median.len());
    }
    println!("Http Codes Present     :");
    for (code, num_times) in &response_codes {
        println!("\tCode: {}, Occurances: {}", code, num_times);
    }
}

/// Parses a URL into its root and path
/// 
/// Args:
/// 
///     site: the full URL
/// 
/// Ret:
/// 
///     url: the root URL
///     path: url path information
fn parsr_url(site: String) -> (String, String) {

    let mut parts : Vec<&str> = site.split("/").collect();
    let url = String::from(parts[0]);
    parts.remove(0);
    let path = String::from(parts.join("/"));

    return (url, path)
}

/// Initiates a connection to a URL and sends a get request
/// Times the run and returns basic info about each connection
/// 
/// Args:
/// 
///     url: the root URL
///     path: url path information
/// 
/// Ret:
/// 
///     res: A result struct containing an HttpReturn struct
fn connect(url: String, path: String) -> Result<HttpReturn> {

    let now = Instant::now();
    let url = format!("{}:80", url);
    let mut stream = TcpStream::connect(url.clone())?;

    // Creates the HTTP Request
    let mut request_data = String::new();
    request_data.push_str(&format!("GET /{} HTTP/1.0", path));
    request_data.push_str("\r\n");
    request_data.push_str(&format!("Host: {}", url));
    request_data.push_str("\r\n");
    request_data.push_str("Connection: close");
    request_data.push_str("\r\n");
    request_data.push_str("\r\n");

    // Writes the request and reads back the response.
    // This section is timed, but parsing the response is not.
    let _ = stream.write_all(request_data.as_bytes())?;
    let mut buf = String::new();
    let _ = stream.read_to_string(&mut buf)?;
    let total_time: u128 = now.elapsed().as_nanos();
    let split = buf.split("\r\n");
    let res: Vec<&str> = split.collect();
    let (headers, body) = split_results(res);
    let response_size = buf.into_bytes().len();

    let http_ret: HttpReturn = HttpReturn{ headers, body, total_time, response_size };
    Ok(http_ret)
}

/// Splits up a returned Http response into the headers and the body
/// 
/// Args:
/// 
///     res: a vector or strings representing each line of the http response
/// 
/// Ret:
/// 
///     headers: vec containing each line of the headers
///     body: string of the body of the response
fn split_results(res: Vec<&str>) -> (Vec<String>, String) {

    let mut headers: Vec<String> = Vec::new();
    let mut body = String::new();
    let mut breaker: bool = false;

    for line in res.into_iter() {
        if line.is_empty() && !breaker {
            breaker = true;
            continue;
        }

        if !breaker {
            headers.push(line.to_string());
        } else {
            body.push_str(&line);
        }
    }

    return (headers, body);
}
