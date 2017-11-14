#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_bencode;
extern crate serde_bytes;
extern crate base64;

use serde_bencode::de;
use serde_bytes::ByteBuf;
use std::io::{self, Read};
use std::env;
use std::fs::File as F;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TorrentError {
    UnsupportedType,
    InvalidPiecesLength,
    Other(Box<Error>),
}

impl fmt::Display for TorrentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str( Error::description(self) )
    }
}

impl Error for TorrentError {
    fn description(&self) -> &str {
        match *self {
            TorrentError::UnsupportedType  => "This type is not supported yet",
            TorrentError::InvalidPiecesLength  => "The length of the pieces must be dividable by 20",
            TorrentError::Other(ref e)     => e.description(),
        }
    }
}
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RangedFile{
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    file_pieces: FilePieces,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Info {
    name: String,
    pieces: ByteBuf,
    #[serde(rename="piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<File>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename="root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Outfo {
    name: String,
    pieces: Vec<String>,
    #[serde(rename="piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<RangedFile>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename="root hash")]
    root_hash: Option<String>,
}

impl Outfo{
    fn new(i : Info, files : Vec<RangedFile>)->Self{
        Outfo{
            name: i.name,
            pieces : split_pieces(i.pieces),
            piece_length : i.piece_length,
            md5sum : i.md5sum,
            length : i.length,
            files : Some(files),
            private : i.private,
            path : i.path,
            root_hash : i.root_hash
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FilePieces{
    first_start : PiecePoint,
    last_end : PiecePoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PiecePoint{
    piece : i64,
    byte_point : i64
}

fn split_pieces(bf : ByteBuf)->Vec<String>{
    let mut out : Vec<String> = Vec::new();
    let mut i : usize = 0;
    let mut j : usize = 19;
    let mut k = 1;
    while(i < bf.len()){
        let piece = base64::encode(&bf[i..j]);
        let formatted = format!("{}{}{}" ,k,": ",piece);
        out.push(formatted);
        i+=20;
        j+=20;
        k+=1;
    }
    out
}

fn separate_files(info : &Info) -> Result<Outfo, TorrentError>{
    let mut rfiles : Vec<RangedFile> = Vec::new();
    let mut start : i64 = 0;
    let mut start_byte : i64 = 0;
    let mut actual_byte = 0;
    let mut piece_index : i64 = 0;

    let piece_length : i64 = info.piece_length;
    let pieces_bytes = info.pieces.len();
    if (&pieces_bytes%20!=0){return Err(TorrentError::InvalidPiecesLength)};

    let files = info.files.clone().unwrap();
    let mut iter = IntoIterator::into_iter(files);

    loop {
        match iter.next() {
            Some(file) => {
                let startp = PiecePoint{piece: piece_index+1, byte_point: (start_byte%piece_length)+1};
                actual_byte = start_byte+file.length;
                piece_index = actual_byte/piece_length;

                let endp = PiecePoint{piece: piece_index+1, byte_point: (actual_byte%piece_length)};
                start = piece_index;
                start_byte = actual_byte;

                let rf = RangedFile{
                    path: file.path,
                    length: file.length,
                    md5sum: file.md5sum,
                    file_pieces: FilePieces{
                        first_start : startp,
                        last_end : endp,
                    },
                };
                rfiles.push(rf);

            },
            None => break,
        }
    }
    Ok(Outfo::new(info.clone(), rfiles))
}

#[derive(Debug, Serialize ,Deserialize)]
struct Torrent {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename="announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename="creation date")]
    creation_date: Option<i64>,
    #[serde(rename="comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename="created by")]
    created_by: Option<String>,
}

#[derive(Debug, Serialize ,Deserialize)]
struct JsonTorrent {
    info: Outfo,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename="announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename="creation date")]
    creation_date: Option<i64>,
    #[serde(rename="comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename="created by")]
    created_by: Option<String>,
}

impl JsonTorrent{
    fn new(t : Torrent)->Self{
        JsonTorrent{
            info : separate_files(&t.info).unwrap(),
            announce : t.announce,
            nodes : t.nodes,
            encoding : t.encoding,
            httpseeds : t.httpseeds,
            announce_list : t.announce_list,
            creation_date : t.creation_date,
            comment : t.comment,
            created_by : t.created_by
        }
    }
}

fn render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let &Some(ref al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    println!("path:\t\t{:?}", torrent.info.path);
    if let &Some(ref files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
    println!("pieces: {:?}", base64::encode(&torrent.info.pieces));
}

fn call(input : String, output : String){
    let mut file = F::open(&input);
    if let Err(e) = file{println!("{:?}", "           File not found");std::process::exit(666)}
    let mut readable = file.unwrap();

    let mut contents = Vec::new();
    readable.read_to_end(&mut contents);

    match de::from_bytes::<Torrent>(&contents) {
        Ok(t) => {
            let torrent = JsonTorrent::new(t);
            let json : String = serde_json::to_string_pretty(&torrent).unwrap();
            let mut out = F::create(&output).unwrap();
            out.write_all(json.as_bytes()).unwrap();
            },
        Err(e) => println!("ERROR: {:?}", e),
    }
}

fn main() {
    let mut args = env::args();
    match env::args().count(){
        2 => {
            let opt1 = args.nth(1).unwrap_or_default();
            let opt2 = String::from("gazorpazorp.burp");
            println!("
            Input file: {}
            Output file: {}", &opt1, &opt2);
            call(opt1, opt2);
            println!("{:?}", "Done!");
        }
        3 => {
            let opt1 = args.nth(1).unwrap_or_default();
            let opt2 = args.nth(0).unwrap_or_default();
            println!("
            Input file: {}
            Output file: {}", &opt1, &opt2);
            call(opt1, opt2);
            println!("{:?}", "Done!");
        },
        _ => {
            println!("
            1. parameter is the name of the input file (.torrent)
            2. parameter is the name of the output file, entirelly your choice
            If no output parameter were given, output will be named \"gazorpazorp.burp\"
            ");
        },

    }

}
