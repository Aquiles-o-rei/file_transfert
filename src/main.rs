use std::net::{TcpListener, TcpStream};
use std::{fs, io, thread};
use std::io::{Read, Write, Error, BufReader, BufRead, stderr};
use std::env;
use std::fs::{File, metadata};
use std::fs::OpenOptions;
use std::path::{Path};
extern crate walkdir;
 use std::string::String;
extern crate base64;

use base64::{encode, decode};
use users::get_current_username;
use walkdir::WalkDir;
use chrono::{DateTime, Utc};
use std::ffi::OsStr;
use std::fmt::format;
use pbr::{ProgressBar, Units};
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::de::Unexpected::Str;
use tar::Unpacked::File as taFile;


fn main() {

    let args: Vec<String> = env::args().collect();
    parse(args);


}



fn parse(args : Vec<String>)
{

    if args.len()<2
    {
        panic!("not enough arguments");
    }

    else {
        let first = &args[1].clone();

        // println!("{}",first);
        if first =="--server"
        {
            on_server();
        }
        else if first =="--client"
        {

         let _ =   on_client();
        }

    }
}


fn on_server()
{
    let listener = TcpListener::bind("0.0.0.0:8888").expect("could not bind");
    for stream in listener.incoming()
    {
        match stream {
            Ok(stream) => {
                thread::spawn( move ||{
                    handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
            Err(e) => {eprintln!("failed {}",e);}
        };
    }

}
fn handle_client(mut stream : TcpStream) -> Result<(),Error>
{
    println!("incoming connection from {}", stream.peer_addr()?);
   // let mut buf = [0; 512];
    loop {
        /*  let bytes_read = stream.read(&mut buf)?;
        if bytes_read ==0 { return  Ok(())}
        stream.write(&buf[..bytes_read])?;*/


        let mut input = String::new();

        println!("enter a file or a directory path :");
        io::stdin().read_line(&mut input).expect("failed to read from sdin");


        if !input.is_empty()
        {
            input = input.trim().parse().unwrap();
            let b = &input.as_str();
            let path = Path::new(b);
            
            
            
            if path.is_file() {
                println!("file");
                let filename = path.file_name();
                let mut buffer = [0u8; 512];
                match filename.and_then(OsStr::to_str) {
                    None => {}
                    Some( nu) => {
                        let na = format!("{}\n", nu);
                        stream.write(na.as_bytes()).expect("failed to write to client");
                        println!("sending file");
                    }
                }


                let mut file = File::open(&input)?;
                let meta = metadata(&input)?;
                let size = meta.len();

                let a = format!("{}\n", size);
                stream.write(&a.as_bytes()).expect("failed to write to client");

                loop {
                    let read_count = file.read(&mut buffer)?;
                    stream.write(&buffer[..read_count]).expect("failed to write to client");


                    if read_count != 512 {
                        break;
                    }
                }
            }
            else if path.is_dir() {
                println!("dir");
                // it is a directory we'll iterate recursively over it a send it


                let filename = path.file_name();

                let mut ziped = String::new();
                match filename.and_then(OsStr::to_str) {
                    None => {}
                    Some( nu) => {

                        let  compressed =  compress(nu,&input);
                        ziped   =  match compressed {
                            Ok(ne) => ne,
                            Err(_) => {"".to_string()}
                        };
                        let na = format!("{}\n",ziped);
                        stream.write(na.as_bytes()).expect("failed to write to client");
                        println!("sending file");


                    }
                }







                println!("the absolute file path is {}",&ziped);



                match File::open(&ziped) {
                    Ok(mut file) => {

                        let meta = metadata(&ziped)?;
                        let size = meta.len();

                        let a = format!("{}\n", size);

                        stream.write(&a.as_bytes()).expect("failed to write the file size to client ");

                        loop {
                            let mut buffer = [0u8; 512];
                            let  read_count  = file.read(&mut buffer)?;
                            stream.write(&buffer[..read_count]).expect("failed to write to client");
                            if read_count != 512 {

                                deletef(ziped);
                                break;
                            }
                        }

                    },
                    Err(e) => {eprintln!("failed to open {}",e);}
                } ;




                                        //now  send

            }
        }

    }
}

       fn on_client() -> Result<(), Box<dyn std::error::Error>>
        {
            let mut buffer: Vec<u8> = Vec::new();
            let  stream = TcpStream::connect("127.0.0.1:8888").expect("could not connect to server");
            let mut filename: &str = "";
            let mut size: u64 = 0;
            let mut buf: Vec<u8> = Vec::new();
            let mut progress: usize = 0;
           // let mut folder = "/home/aquiles/Downloads/FileTransfer";

            let mut folder =String::new();
            let  name  ;

            match get_current_username() {
                Some(uname) => {
                    name = uname.to_str().unwrap();
                    folder = format!("/home/{}/Downloads/FileTransfer", name);
                  let _ =  fs::create_dir(&folder);



                },
                None        => println!("The current user does not exist!"),
            }


            loop {




                /*let mut input = String::new();

                println!("do you want to send  or receive ? y/n:");
                io::stdin().read_line(&mut input).expect("failed to read from sdin");*/





                if filename.is_empty()
                {
                    let mut reader = BufReader::new(&stream);
                    let _ = reader.read_until(b'\n', &mut buffer);

                  let _=  &buffer.pop();



                    filename = std::str::from_utf8(&buffer)?;

                    println!(" receiving {}", &filename);


                    continue;
                }
                if size == 0 {
                    let mut reader = BufReader::new(&stream);
                    let _ = reader.read_until(b'\n', &mut buf);


                      size = std::str::from_utf8(&buf)?.trim().parse()?;


                    println!(" the file size is {}", size);
                    continue;
                }

                let mut pb = ProgressBar::on(stderr(), size);
                pb.set_units(Units::Bytes);


                /*if !filename.is_empty()
        {
            stream.write(input.as_bytes()).expect("failed to write to client");

        }*/

                if !filename.is_empty() {
                    let mut reader = BufReader::new(&stream);

                //    let mut file = File::create(format!("ok{}", filename))?;


                    let mut vec: Vec<u8> = Vec::new();

                    loop {
                        let mut bouffe = [0u8; 512];
                        let read_count = reader.read(&mut bouffe);
                        let mut count: usize = 0;
                        match read_count {
                            Ok(num) => {
                                count = num;
                                // println!("{}", count);
                            }
                            Err(e) => { eprintln!(" error {}", e) }
                        }


                        vec.append(&mut bouffe[..count].to_vec());
                        progress += 512;
                        pb.add(512);

                        if count != 512 {
                            println!("the vector size is {}", vec.len());

                           // file.write(&vec).expect("unable to write file");

                            let complete = format!("{}/{}", &folder, &filename);
                            println!("{}", &complete);
                            let path: &Path = Path::new(complete.as_str());
                            println!("finishing up .. please wait ");
                            fs::write(path, &vec).expect("failed to write file");
                            progress = 0;
                            size = 0;
                            filename = "";
                            vec.clear();
                            buf.clear();
                            buffer.clear();
                            println!("finished");
                            break;
                        }
                    }
                }





            }
        }





fn compress(name : &str, src : &str) -> Result<String, std::io::Error>
{
    println!("preraring your files...");

    let filename = format!("{}.tar.gz",name);
    let tar_gz = File::create(filename.clone().as_str())?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(name, src)?;

    Ok(filename)
}


fn deletef(path: String)
{
    match fs::remove_file(path)  {
        Ok(_) => {println!("Saved successfully !");}
        Err(_) => {println!("an error occured!");}
    } ;
}

/*fn create() -> String
{
    let  name  ;
    let mut foder  = "".to_string();
    match get_current_username() {
        Some(uname) => {
            name = uname.to_str();
           foder = format!("/home/{}/Downloads/transferas",&name.unwrap());
            fs::create_dir(&foder);

        },
        None        => println!("The current user does not exist!"),
    }

    foder

}
*/