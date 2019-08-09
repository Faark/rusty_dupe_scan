use std::fs;
use std::fs::{File, DirEntry};
use std::io::{BufReader, Read, Error, ErrorKind};
use sha2::{Sha256, Digest};
use std::collections::HashMap;


type Result<T> = std::io::Result<T>;
//type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// File -> Sha256 bytes
fn hash_file(file: File) -> Result<[u8; 32]>{
    println!("hashing");
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();


    let mut buffer = [0; 1024 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0{
            break;
        }
        hasher.input(&buffer[..count]);
    }
    Ok(hasher.result().into())
}

struct DupeScanItem {
    // name of this file/dir
    name: String,

    // id of the parent. Own id for roots
    parent_id: usize,
}
struct DupeScan {
    // list of all dirs, grows as we find new ones, index is ID, is a directed graph with nodes always having larger IDs than parents
    dirs: Vec<DupeScanItem>,

    // list of all files, grouped by hashes
    files: HashMap<[u8; 32], Vec<DupeScanItem>>,
}
impl DupeScan {
    pub fn new() -> DupeScan {
        DupeScan{
            dirs: vec![],
            files: HashMap::new(),
        }
    }

    fn try_scan_entry(&mut self, entry: DirEntry, parent_id: usize) -> Result<()> {
        let info = entry.metadata()?;
        let current_item = DupeScanItem {
            name: entry.file_name().to_string_lossy().to_string(),
            parent_id
        };

        println!("scanning {}", current_item.name);

        if info.is_file() {
            println!("file");
            let file = File::open(entry.path())?;
            let hash = hash_file(file)?;

            fn new_vec() -> Vec<DupeScanItem> {
                Vec::with_capacity(1)
            }
            let hash_files = self.files.entry(hash).or_insert_with(new_vec);

            hash_files.push(current_item);

            Ok(())
        }else if info.is_dir(){
            println!("dir");
            let dir = fs::read_dir(entry.path())?;

            let current_id = self.dirs.len();
            self.dirs.push(current_item);

            println!("dir-loop");
            for sub_entry in dir {
                println!("dir-loop-inside");
                self.scan_entry(sub_entry, current_id);
            }
            Ok(())
        }else{
            println!("scary");
            let err_msg = format!("Scary type, skipping path {:?}", entry.path());
            Err(Error::new(ErrorKind::InvalidData, err_msg))
        }
    }

    // scans an dir or file, adding it to the local db or skipping it in case of error
    fn scan_entry(&mut self, entry: Result<DirEntry>, parent_id: usize) {
        println!("scan_entry");

        if let Ok(entry) = entry {
            let name = entry.path();
            if let Err(err) = self.try_scan_entry(entry, parent_id) {
                println!("Failed at {:?} with {:?}", name, err);
            }else{
                println!("Done {:?}.", name);
            }
        }else{
            println!("TODO: BETTER ERROR MSG # 65454168");
        }
    }


    pub fn scan_root(&mut self, root_path: String) -> Result<()> {
        let dir = fs::read_dir(&root_path )?;
        let root_id = self.dirs.len();
        println!("Start scanning {:?}", &root_path);
        let item = DupeScanItem{
            name: root_path,
            parent_id: root_id,
        };
        self.dirs.push(item);

        for entry in dir {
            self.scan_entry(entry, root_id );
        }

        Ok(())
    }
}



fn main() -> Result<()> {
    let drive_letters = ["C","D","E","G","H"];
    let mut scanner = DupeScan::new();

    for root_letter in drive_letters.iter() {
        //let root_path = root_letter + r":\\";
        let root_path = [root_letter, r":\\"].join("");

        scanner.scan_root(root_path)?;
    }

    println!("Done.");
    Ok(())
}
