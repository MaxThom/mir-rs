use arrow::error::ArrowError;
use arrow::ipc::{reader::FileReader};
use arrow::record_batch::RecordBatch;

use std::fs::File;
use std::sync::Arc;
use arrow::datatypes::Schema;

struct ArrowReader {
    file_reader: FileReader<File>,
    batches: Option<Vec<arrow::record_batch::RecordBatch>>,
    schema: Option<Arc<Schema>>,
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

    pub fn try_open_file(file_path: &str) -> Result<Self, ArrowError> {
        let file = match File::open(file_path) {
            Ok(file) => file.try_clone().unwrap(),
            Err(err) => return Err(ArrowError::IoError(err.to_string())),
        };

        let file_reader = match FileReader::try_new(file, None) {
            Ok(file) => file,
            Err(err) => return Err(err),
        };

        println!("number of batches: {:?}", file_reader.num_batches());

        Ok(Self {
            file_reader: file_reader,
            batches: None,
            schema: None,
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
    println!("Hello, world!");

    let rdr = match ArrowReader::try_open_file(&"../arrow-generator/data/gen.arrow".to_string())  {
        Ok(rdr) => rdr,
        Err(err) => panic!("{:?}", err),
    };

    //let res = match rdr.open_file("../arrow-generator/data/gen.arrow".to_string()) {
    //    Ok(file) => file,
    //    Err(err) => panic!("{:?}", err),
    //};

    let mut batches: Vec<arrow::record_batch::RecordBatch> = vec![];
    for i in rdr {
        let batch = match i  {
            Ok(batch) => batch,
            Err(err) => return println!("{}", err.to_string()),
        };

        println!("cols: {:?}, rows: {}", batch.num_columns(), batch.num_rows());
        println!("{:?}", batch);
        batches.push(batch);
    }

    //let res = match read_from_arrow("../arrow-generator/data/gen.arrow".to_string()) {
    //    Ok(file) => file,
    //    Err(err) => panic!("{:?}", err),
    //};

    //for i in ArrowReader().next() {
    //    println!("> {}", i);
    //}

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