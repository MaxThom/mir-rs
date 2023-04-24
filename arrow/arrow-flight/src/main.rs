use arrow::error::{ArrowError}; //, Result as ArrowResult
use arrow::ipc::{reader::FileReader};
use arrow::record_batch::RecordBatch;

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use clap::Parser;

#[derive(Parser)]
#[command(author = "MaxThom", version, about = "A simple arrow reader fly the records to arrow-flux")]
struct Cli {
    #[arg(short, long, default_value_t = String::from("../arrow-generator/data/gen.arrow"))]
    path: String,
}

impl Display for Cli {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "File: {}", self.path)
    }
}


struct ArrowReader {
    file_reader: FileReader<File>,
}

impl ArrowReader {
    //pub fn new() -> Self {
    //    Self {
    //        file: None,
    //        file_reader: None,
    //        batches: None,
    //        schema: None,
    //    }
    //}

    pub fn try_open_file(file: File) -> Result<Self, ArrowError> {
        let file_reader = match FileReader::try_new(file, None) {
            Ok(file) => file,
            Err(err) => return Err(err),
        };

        println!("number of batches: {:?}", file_reader.num_batches());

        Ok(Self {
            file_reader: file_reader,
        })
    }
}

impl Iterator for ArrowReader {
    type Item = Result<RecordBatch, ArrowError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.file_reader.next()
    }
}

fn main() {
    let args = Cli::parse();
    println!("Hello, arrow-flight!");
    println!("---");
    println!("{}", args);
    println!("---");

    let rdr = match read_from_arrow(args.path) {
        Ok(rdr) => rdr,
        Err(err) => {
            println!("{}", err.to_string());
            panic!();
        }
    };

    let mut batches: Vec<arrow::record_batch::RecordBatch> = vec![];
    for i in rdr {
        let batch = match i  {
            Ok(batch) => batch,
            Err(err) => return println!("{}", err.to_string()),
        };

        println!("cols: {:?}, rows: {}", batch.num_columns(), batch.num_rows());
        batches.push(batch);
    }

}

fn read_from_arrow(file_path: String) -> Result<ArrowReader, ArrowError> {
    let file = match File::open(file_path) {
        Ok(file) => file.try_clone().unwrap(),
        Err(err) => return Err(ArrowError::IoError(err.to_string())),
    };

    ArrowReader::try_open_file(file)
}


//fn read_from_arrow(file_path: String) -> Result<Vec<arrow::record_batch::RecordBatch>, String> {
//    let file = match File::open(file_path) {
//        Ok(file) => Some(file),
//        Err(err) => return Err(err.to_string()),
//    };
//
//    let rdr = match FileReader::try_new(file, None) {
//        Ok(file) => Some(file),
//        Err(err) => return Err(err.to_string()),
//    };
//
//
//    let mut batches: Vec<arrow::record_batch::RecordBatch> = vec![];
//    for i in rdr {
//        let batch = match i  {
//            Ok(batch) => batch,
//            Err(err) => return Err(err.to_string()),
//        };
//
//        println!("cols: {:?}, rows: {}", batch.num_columns(), batch.num_rows());
//        println!("{:?}", batch);
//        batches.push(batch);
//    }
//
//    Ok(batches)
//}