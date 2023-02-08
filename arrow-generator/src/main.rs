use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{Write};
use std::sync::Arc;
use std::time::Instant;

use arrow::array::*;
use arrow::csv;
use arrow::datatypes::*;
use arrow::error::{ArrowError, Result as ArrowResult};
use arrow::json;
use arrow::record_batch::*;

use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;

use clap::Parser;

// Parquet Reader
// https://parquetreader.com/home

type TypeGenerator = fn(&SchemaArgs, usize) -> ArrayRef;

#[derive(Parser)]
#[command(author = "MaxThom", version, about = "A Very simple arrow generator")]
struct Cli {
    #[arg(short, long, default_value_t = String::from("timestamp:date(2020-02-06 02:03:34.722919226 UTC,600);name:string(4);family:string(8);age:int(0,100);grade:float(0,20)"))]
    schema: String,

    #[arg(short, long, default_value_t = String::from("./gen"))]
    path: String,

    #[arg(short, long, default_value_t = 100)]
    count: usize,

    #[arg(short, long, default_value_t = String::from("arrow"))]
    output: String,
}

struct SchemaCsv {
    schema: Schema,
    args: HashMap<String, SchemaArgs>,
}

struct SchemaArgs {
    i32_from: Option<i32>,
    i32_to: Option<i32>,
    f32_from: Option<f32>,
    f32_to: Option<f32>,
    length: Option<usize>,
    interval_sec: Option<i32>,
    date_from: Option<DateTime<Utc>>,
}

fn main() {
    let args = Cli::parse();
    println!("Hello, csv-generator!");
    println!("---");
    println!("{}", args);
    println!("---");
    let start = Instant::now();

    let schema = match parse_schema_from_string(args.schema) {
        Ok(schema) => schema,
        Err(err) => {
            println!("{:?}", err.to_string());
            panic!();
        }
    };

    let record_batch = match generate_records_from_schema(schema, args.count) {
        Ok(batch) => batch,
        Err(err) => {
            println!("{:?}", err.to_string());
            panic!();
        }
    };

    match write_to_ouput(record_batch, args.output, args.path) {
        Ok(res) => {
            println!("{:?}", res);
        }
        Err(err) => {
            println!("{:?}", err.to_string());
            panic!();
        }
    }

    println!("Time elapsed: {:.2}s", start.elapsed().as_secs_f32());
    println!("---");
    println!("Au revoir !");
}

fn parse_schema_from_string(str_schema: String) -> ArrowResult<SchemaCsv> {
    let mut args: HashMap<String, SchemaArgs> = HashMap::new();
    let mut fields: Vec<Field> = vec![];
    // <field_name>:<field_type>;...
    // replace(" ", "")
    for col in str_schema.split(';') {
        // Split name and type and args
        let details = match col.split_once(':') {
            Some(x) => x,
            None => {
                return Err(ArrowError::ParseError(format!(
                    "{}{}{}",
                    "Can't parse ", col, ". See supported datatype (--help)."
                )))
            }
        };
        let name = details.0;
        let field = details.1;

        // Split type and args
        let field: Vec<&str> = field.split('(').collect();
        let field_type = match field.get(0) {
            Some(x) => *x,
            None => {
                return Err(ArrowError::ParseError(format!(
                    "{}{}{}",
                    "Can't parse ", col, ". See supported datatype (--help)."
                )))
            }
        };
        let field_args = match field.get(1) {
            Some(x) => x.trim_end_matches(')'),
            None => {
                return Err(ArrowError::ParseError(format!(
                    "{}{}{}",
                    "Can't parse ", col, ". See supported datatype (--help)."
                )))
            }
        };

        // get string type to Arrow type
        let arrow_field_type = match field_type {
            "int" => DataType::Int32,
            "float" => DataType::Float32,
            "string" => DataType::Utf8,
            "date" => DataType::Timestamp(TimeUnit::Second, Some(String::from("+00:00"))),
            _ => {
                return Err(ArrowError::ParseError(format!(
                    "{}{}{}",
                    "Can't parse ", field_type, ". See supported datatype (--help)."
                )))
            }
        };
        fields.push(Field::new(name, arrow_field_type, false));

        // get args
        let field_arg_list: Vec<&str> = field_args.split(',').collect();
        let arrow_field_args: SchemaArgs = match field_type {
            "int" => SchemaArgs {
                i32_from: Some(field_arg_list[0].parse().unwrap()),
                i32_to: Some(field_arg_list[1].parse().unwrap()),
                f32_from: None,
                f32_to: None,
                length: None,
                interval_sec: None,
                date_from: None,
            },
            "float" => SchemaArgs {
                i32_from: None,
                i32_to: None,
                f32_from: Some(field_arg_list[0].parse().unwrap()),
                f32_to: Some(field_arg_list[1].parse().unwrap()),
                length: None,
                interval_sec: None,
                date_from: None,
            },
            "string" => SchemaArgs {
                i32_from: None,
                i32_to: None,
                f32_from: None,
                f32_to: None,
                length: Some(field_arg_list[0].parse().unwrap()),
                interval_sec: None,
                date_from: None,
            },
            "date" => SchemaArgs {
                i32_from: None,
                i32_to: None,
                f32_from: None,
                f32_to: None,
                length: None,
                interval_sec: Some(field_arg_list[1].parse().unwrap()),
                date_from: Some(field_arg_list[0].parse().unwrap()),
            },
            _ => {
                return Err(ArrowError::ParseError(format!(
                    "{}{}{}",
                    "Can't parse ", field_type, ". See supported datatype (--help)."
                )))
            }
        };
        args.insert(name.to_string(), arrow_field_args);
    }

    Ok(SchemaCsv {
        schema: Schema::new(fields),
        args: args,
    })
}

fn generate_records_from_schema(schema: SchemaCsv, count: usize) -> ArrowResult<RecordBatch> {
    let mut generator_functions: HashMap<&DataType, TypeGenerator> = HashMap::new();
    generator_functions.insert(&DataType::Utf8, generate_string);
    generator_functions.insert(&DataType::Int32, generate_int);
    generator_functions.insert(&DataType::Float32, generate_float);
    let binding = DataType::Timestamp(TimeUnit::Second, Some(String::from("+00:00")));
    generator_functions.insert(&binding, generate_date);

    let mut columns: Vec<ArrayRef> = Vec::new();
    for field in schema.schema.all_fields() {
        let x = schema.args.get(field.name()).unwrap();
        columns.push(generator_functions.get(field.data_type()).unwrap()(
            x, count,
        ));
    }

    // Build recoard batch by combining schema and data
    let batch = RecordBatch::try_new(Arc::new(schema.schema), columns)?;

    Ok(batch)
}

fn generate_string(args: &SchemaArgs, count: usize) -> ArrayRef {
    let mut array: Vec<String> = vec![String::new(); count];
    for i in 0..count {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(args.length.unwrap())
            .map(char::from)
            .collect();
        array[i] = s;
    }

    Arc::new(StringArray::from(array))
}

fn generate_int(args: &SchemaArgs, count: usize) -> ArrayRef {
    let mut rng = rand::thread_rng();

    let mut array: Vec<i32> = vec![0; count];
    for i in 0..count {
        array[i] = rng.gen_range(args.i32_from.unwrap()..args.i32_to.unwrap());
    }

    Arc::new(Int32Array::from(array))
}

fn generate_float(args: &SchemaArgs, count: usize) -> ArrayRef {
    let mut rng = rand::thread_rng();

    let mut array: Vec<f32> = vec![0.0; count];
    for i in 0..count {
        array[i] = rng.gen_range(args.f32_from.unwrap()..args.f32_to.unwrap());
    }

    Arc::new(Float32Array::from(array))
}

fn generate_date(args: &SchemaArgs, count: usize) -> ArrayRef {
    let now: i64 = args.date_from.unwrap().timestamp(); //Utc::now().timestamp();
    let interval: i64 = args.interval_sec.unwrap() as i64;
    let mut array: Vec<i64> = vec![now; count];
    for i in 0..count {
        array[i] = array[i] + interval * i64::try_from(i).unwrap();
    }

    Arc::new(TimestampSecondArray::from(array).with_timezone("+00:00".to_string()))
}

fn write_to_ouput(batch: RecordBatch, output: String, file_name: String) -> Result<String, String> {
    match output.as_str() {
        "csv" => write_to_csv(batch, file_name),
        "json" => write_to_json(batch, file_name),
        "arrow" => write_to_arrow(batch, file_name),
        "parquet" => write_to_parquet(batch, file_name),
        _ => Err(format!("{} is not supported.", output)),
    }
}

fn write_to_csv(batch: RecordBatch, file_name: String) -> Result<String, String> {
    let file = match File::create(format!("{}.csv", file_name)) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    let mut writer = csv::Writer::new(file);
    writer.write(&batch).unwrap();

    Ok(format!(
        "{} records written to {}.",
        batch.num_rows(),
        format!("{}.csv", file_name)
    ))
}

fn write_to_json(batch: RecordBatch, file_name: String) -> Result<String, String> {
    let mut file = match File::create(format!("{}.json", file_name)) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    let json_rows = json::writer::record_batches_to_json_rows(&[batch.clone()]).unwrap();
    match writeln!(file, "{:?}", json_rows) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    Ok(format!(
        "{} records written to {}.",
        batch.num_rows(),
        format!("{}.json", file_name)
    ))
}

fn write_to_arrow(batch: RecordBatch, file_name: String) -> Result<String, String> {
    let file = match File::create(format!("{}.arrow", file_name)) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    let mut writer = match arrow::ipc::writer::FileWriter::try_new(file, &batch.schema()) {
        Ok(x) => x,
        Err(err) => return Err(err.to_string()),
    };

    match writer.write(&batch) {
        Ok(_) => (),
        Err(err) => return Err(err.to_string()),
    };

    match writer.finish() {
        Ok(_) => (),
        Err(err) => return Err(err.to_string()),
    };

    Ok(format!(
        "{} records written to {}.",
        batch.num_rows(),
        format!("{}.arrow", file_name)
    ))
}

fn write_to_parquet(batch: RecordBatch, file_name: String) -> Result<String, String> {

    let file = match File::create(format!("{}.parquet", file_name)) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    let props = WriterProperties::builder()
    //.set_compression(Compression::SNAPPY)
    .set_writer_version(parquet::file::properties::WriterVersion::PARQUET_2_0)
    //.set_encoding(parquet::basic::Encoding::PLAIN_DICTIONARY)
    .build();

    let mut writer = match ArrowWriter::try_new(file, batch.schema(), Some(props)) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };
   
    match writer.write(&batch) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };
    
    match writer.close() {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    Ok(format!(
        "{} records written to {}.",
        batch.num_rows(),
        format!("{}.parquet", file_name)
    ))
}

impl Display for Cli {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "File: {}\nCount: {}\nSchema: {}",
            self.path, self.count, self.schema
        )
    }
}
