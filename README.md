# Web Site Profiler

## Build Instructions

This CLI tool uses cargo to build, and has additional command line args parsed by [clap](https://crates.io/crates/clap).

Clap requires a --url / -u args that specifies the URL to be tested.
There is an additonal --profile / -p arg that can be passed that will allow for the tool to run multiple times.

Functionality is a bit different when there is no profile set.

Examples:
    cargo run -- -u 2020-general-assignment.palex.workers.dev -p 10
    cargo run -- -u 2020-general-assignment.palex.workers.dev

## Findings

Since I used Rust's low level TCP socket stream, instead of a third party HTTP package, this tool is unable to handle HTTPS requests. It seemed like an unfeaable amount of work to implement SSL/TLS connection on top of my tool for a project this size. Since the assignment specifically says not to use a external package, I opted to keep the implementation this way.

Because of this, any HTTP request to a site that required HTTPS would give a `301 Moved Permanently` response code that directs to the HTTPS version of the site. Any call to a HTTPS (`https://www.cloudflare.com/` instead of `www.cloudflare.com/`) would result in the TCPStream throwing an error.

Testing against the site form the first part of the assignement would return the correct page, as the worker sites do not require HTTPS, but do have an HTTPS option.
