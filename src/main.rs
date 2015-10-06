extern crate mioco;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;

use std::net::SocketAddr;
use std::str::FromStr;
use std::str;
use std::io::{Read, Write};
use mioco::mio::tcp::{TcpSocket};
use docopt::Docopt;

const USAGE: &'static str = "
TCP Slammer - Not a virus. I swear.

Usage:
  slammer listen <address>
  slammer connect <address> <clients> <iterations>

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_address: String,
    arg_clients: Option<u32>,
    arg_iterations: Option<u32>,
    cmd_listen: bool,
    cmd_connect: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    env_logger::init().unwrap();

    if args.cmd_listen {
        server(args.arg_address);
    }
    else if args.cmd_connect {
        client(args.arg_address, args.arg_clients.unwrap(), args.arg_iterations.unwrap());
    }
    else {
        println!("{}", USAGE);
    }
}

fn server(addr : String) {

    mioco::start(move |mioco| {

        let sock = try!(TcpSocket::v4());
        try!(sock.bind(&toaddr(&addr)));
        let sock = try!(sock.listen(1024));

        println!("Starting tcp echo server on {:?}", sock.local_addr().unwrap());

        let sock = mioco.wrap(sock);
        let mut count = 0;
        loop {
            let conn = sock.accept().unwrap();

            count += 1;
            println!("accepted {}", count);
            mioco.spawn(move |mioco| {
                let mut conn = mioco.wrap(conn);

                let mut buf = [0u8; 128];
                loop {
                    let size = conn.read(&mut buf).unwrap();
                    if size == 0 {
                        break;
                    }
                    conn.write_all(&mut buf[0..size]).unwrap();
                }

                Ok(())
            });
        }
    });
}

fn client(addr : String, num_clients : u32, num_iters : u32) {

    let mut i : u32 = 0;
    
    mioco::start(move |mioco| {

        while i < num_clients {
            let sock = TcpSocket::v4().unwrap();

            let conn = sock.connect(&toaddr(&addr)).unwrap();
            mioco.spawn(move |mioco| {
                let mut conn = mioco.wrap(conn.0);
                let mut j = 0;
                let mut buf = [0u8; 128];
                while j <= num_iters {
                    let sz = {
                        let mut ptr = &mut buf[..];
                        write!(ptr, "{}", j).unwrap();
                        128 - ptr.len()
                    };
                    conn.write(&mut buf[0 .. sz]).unwrap();
                    let sz = conn.read(&mut buf).unwrap();
                    let b = str::from_utf8(&buf[0 .. sz]).unwrap();
                    println!("{} {}", i, b);
                    let ret = b.parse::<u32>().unwrap();
                    assert!(ret == j);
                    j += 1;
                }
                Ok(())
            });
            i += 1;
        }
        Ok(())
    });
}

fn toaddr(addr : &String) -> SocketAddr {
    FromStr::from_str(addr).unwrap()
}
