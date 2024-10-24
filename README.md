# Request shooter 🔫
Is a high performance HTTP benchmarking tool written in rust 🦀.

### Key features
- event driven request sending and response reading, which makes it incredibly fast
- Multithreaded if you want
- Accurate result calculations
- Readable and colorful benchmark statistics in terminal

### Current limitations
- Supported only HTTP version 1.1
- Not supported SSL/TLS connections
- Currently tested only on Fedora Linux OS

### Next goals
- Support latest HTTP versions
- Implement SSL/TLS connections (thinking about rustls lib)
- Support other platforms
- More granular benchmark statistics

# Usage

### Installation
1. clone repository
 ~~~sh
 git clone https://github.com/ilya-4real/req_shooter.git
 ~~~
 2. build shooter with rust compiler
 ~~~sh
 cd req_shooter
 cargo build -r
 ~~~
 3. place binary into any path directory
 ~~~
 cp /target/release/req_shooter ~/.local/bin
 ~~~

 2. launch
 ~~~sh
req_shooter -d 10 -c 50 -t 2 127.0.0.1:8000/
 ~~~
 3. Command line options
 ~~~
Usage: req_shooter [OPTIONS] -d <duration> <url>

Arguments:
  <url>  

Options:
  -t <threads>       how many threads to run [default: 1]
  -c <conns>         how many active connections to use in each thread [default: 100]
  -d <duration>      how long to test in seconds
  -h, --help         Print help
  -V, --version      Print version
 ~~~