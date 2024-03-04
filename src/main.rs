use clap::Parser;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use std::cmp;
use std::env;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_name = "FILE")]
    path: String,

    #[arg(short, long, default_value = "10")]
    max_amount: usize,

    #[arg(short, long, default_value = "eu-central-1")]
    region: String,

    #[arg(short, long)]
    bucket: String,
}

fn get_s3_bucket(args: &Args) -> Bucket {
    let region = args.region.parse().unwrap();
    let access_key = env::var("AWS_ACCESS_KEY_ID").unwrap();
    let secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap();

    let credentials =
        Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap();

    Bucket::new(&args.bucket, region, credentials).unwrap()
}

fn upload(args: &Args, files: &[fs::DirEntry]) {
    let bucket = get_s3_bucket(args);
    let results = bucket
        .list("log".to_string(), Some("/".to_string()))
        .unwrap();

    let mut file_list = Vec::new();

    for result in results {
        for content in result.contents {
            file_list.push((content.key, content.size));
        }
    }

    for path in files {
        let upath = path.path();
        let buffer = fs::read(&upath).unwrap();
        let filename = upath.file_name().unwrap().to_str().unwrap();

        let search_result = file_list.iter().find(|(key, _)| key == filename);

        if Some(search_result) != None {
            let filename = &search_result.unwrap().0;
            let size: usize = search_result.unwrap().1.try_into().unwrap();

            if size >= buffer.len() {
                println!("File already exists: {} ({} bytes)", filename, size);
                continue;
            }

            println!("File already exists but file size is smaller: {}", filename);
        }

        bucket.put_object(filename, &buffer).unwrap();
        println!(
            "Successfully uploaded file: {} ({} bytes)",
            filename,
            buffer.len()
        );
    }
}
fn main() {
    let args = Args::parse();

    let mut paths: Vec<_> = fs::read_dir(&args.path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    paths.sort_by_key(|dir| cmp::Reverse(dir.path()));

    let max_idx = cmp::min(args.max_amount, paths.len());
    upload(&args, &paths[..max_idx]);
}
