extern crate nix;
extern crate base64;
extern crate time;

use nix::sys::termios;

use std::str;
use std::io::Read;
use std::io::prelude::*;
use std::net::{TcpStream, Shutdown};

struct TVConfig<'a> {
    tv_ip:&'a str,
    tv_port:&'a str,
    ip:&'a str,
    mac:&'a str,
    name:&'a str,
    app:&'a str,
    tv_app:&'a str
}

fn convo<'a> (s:&'a str, enc:bool) -> Vec<u8> {
    let enc_s:String;
    let st:&str;

    if enc {
        enc_s = base64::encode(s);
        st = &enc_s;
    } else {
        st = s;
    }

    let len:&[u8] = &[st.as_bytes().len() as u8];
    let zero:&[u8] = &[0];

    [len, zero, st.as_bytes()].concat()
}
 
fn tv_register<'a> (cfg:&'a TVConfig, stream:&'a mut TcpStream) {
    let ip = &convo (cfg.ip, true); 
    let mac = &convo (cfg.mac, true); 
    let name = &convo (cfg.name, true); 
    let app = &convo (cfg.app, false); 

    let hundred:&[u8] = &[100];
    let zero:&[u8] = &[0];

    let msg_bytes:&[u8] = &[hundred, zero, ip, mac, name].concat();
    let msg_str = str::from_utf8 (msg_bytes).unwrap();
    let msg = &convo(msg_str, false);
    
    let chunk:&[u8] = &[zero, app, msg].concat();  
    //println!("{:?}", chunk.len());
    stream.write(chunk);
}

fn tv_send<'a> (cfg:&'a TVConfig, vol:&'a str, stream:&'a mut TcpStream) {
    let cmd = &convo (vol, true); 
    let app = &convo (cfg.tv_app, false);
    
    let zero:&[u8] = &[0];
    
    let msg_bytes:&[u8] = &[zero, zero, zero, cmd].concat();
    let msg_str = str::from_utf8 (msg_bytes).unwrap();
    let msg = &convo(msg_str, false);
    
    let chunk:&[u8] = &[zero, app, msg].concat();  
    //println!("{:?}", chunk.len());
    stream.write(chunk);
}

fn main() {
    let cfg = TVConfig {
                tv_ip : "192.168.1.227",
                tv_port : "55000",
                ip : "127.0.0.1",                
                mac : "00:00:00:00",
                name : "Rust Samsung TVRemote",
                app : "rust.app.samsung",
                tv_app : "rust.tv.remote.samsung"
              };

    let orig_term = termios::tcgetattr(0).unwrap();
    let mut term = termios::tcgetattr(0).unwrap();
    
    term.local_flags.remove(termios::LocalFlags::ICANON);
    term.local_flags.remove(termios::LocalFlags::ISIG);
    term.local_flags.remove(termios::LocalFlags::ECHO);
    
    termios::tcsetattr(0, termios::SetArg::TCSADRAIN, &term).unwrap();
    
    println!("Press Ctrl-C to quit");

    let mut base = time::precise_time_ns();
    println!("{:?}", base);
    
    for byte in std::io::stdin().bytes() {
        let now = time::precise_time_ns();
        if now - base < 150000000 {
            //debounce
            continue;
        }

        let byte = byte.unwrap();
        if byte == 3 {
            break;
        } else if byte == 27 {
            for next_byte in std::io::stdin().bytes() {
                let next_byte = next_byte.unwrap();
                if next_byte == 91 {
                    for next_byte2 in std::io::stdin().bytes() {
                        let next_byte2 = next_byte2.unwrap();
                        if next_byte2 >= 65 && next_byte2 <= 68 { 
                            let res = TcpStream::connect([cfg.tv_ip, cfg.tv_port].join(":"));
                            match res {
                                Ok(mut stream) => {    
                                    tv_register (&cfg, &mut stream);
                                    if next_byte2 & 1 == 0 {
                                        tv_send (&cfg, "KEY_VOLDOWN", &mut stream);
                                    } else {
                                        tv_send (&cfg, "KEY_VOLUP", &mut stream);
                                    }   
                                    stream.shutdown(Shutdown::Both);
                                },
                                _ => println!("failed to connect to tv")
                            }
                        }
                        break;
                    }
                }
                break;
            }
        } else {
            println!("you pressed {}", byte);
        }

        base = time::precise_time_ns();
    }

    println!("Goodbye!");
    termios::tcsetattr(0, termios::SetArg::TCSADRAIN, &orig_term).unwrap();
}