# serial2file
### Usage
Program to save serial data to file and send via TCP simultaneously. 
~~~
serial2file -h
Usage: serial2file [OPTIONS] --output <PATH>

Options:
  -s, --serial <PATH>          Serial port to open [default: /dev/ttyUSB0]
  -b, --baud-rate <BAUD_RATE>  Serial port baud rate [default: 115200]
  -o, --output <PATH>          File to save p1 messages to
  -p, --port <PORT>            TCP port [default: 1080]
  -h, --help                   Print help
  -V, --version                Print version
~~~
### Testing
~~~
# socat -d -d pty,rawer,echo=0 pty,rawer,echo=0
2023/12/02 09:35:10 socat[10387] N PTY is /dev/pts/0
2023/12/02 09:35:10 socat[10387] N PTY is /dev/pts/3
2023/12/02 09:35:10 socat[10387] N starting data transfer loop with FDs [5,5] and [7,7]

# cargo run -- --output /tmp/output --serial /dev/pts/0

# cat data/p1message > /dev/pts/3

# telnet localhost 1080
Trying ::1...
Connection failed: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
/ISk5\2MT382-1000
1-3:0.2.8(50)
0-0:1.0.0(101209113020W)
0-0:96.1.1(4B384547303034303436333935353037)
1-0:1.8.1(123456.789*kWh)
[SNIP]
1-0:41.7.0(02.222*kW)
1-0:61.7.0(03.333*kW)
1-0:22.7.0(04.444*kW)
1-0:42.7.0(05.555*kW)
1-0:62.7.0(06.666*kW)
0-1:24.1.0(003)
0-1:96.1.0(3232323241424344313233343536373839)
0-1:24.2.1(101209112500W)(12785.123*m3)
!EF2F

# tail -f /tmp/test
1-0:21.7.0(01.111*kW)
1-0:41.7.0(02.222*kW)
1-0:61.7.0(03.333*kW)
1-0:22.7.0(04.444*kW)
1-0:42.7.0(05.555*kW)
1-0:62.7.0(06.666*kW)
0-1:24.1.0(003)
0-1:96.1.0(3232323241424344313233343536373839)
0-1:24.2.1(101209112500W)(12785.123*m3)
!EF2F
~~~

### Compilation
~~~
cargo build --release
cargo run --release -- -o /tmp/output -s /dev/ttyUSB0
~~~
