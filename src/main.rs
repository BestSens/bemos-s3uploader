use clap::Parser;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use std::cmp;
use std::env;
use std::fs;
use std::path;

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
	let results = bucket.list("".to_string(), Some("".to_string())).unwrap();

	let mut file_list = Vec::new();

	for result in results {
		for content in result.contents {
			file_list.push((content.key, content.size));
		}
	}

	for path in files {
		let upath = path.path();
		let filename = upath.file_name().unwrap().to_str().unwrap();
		let foldername = upath
			.parent()
			.unwrap()
			.file_name()
			.unwrap()
			.to_str()
			.unwrap();

		let buffer = fs::read(&upath).unwrap();

		let filename = format!("{}/{}", foldername, filename);

		let search_result = file_list.iter().find(|(key, _)| key == &filename);

		if search_result.is_some() {
			let filename = &search_result.unwrap().0;
			let size: usize = search_result.unwrap().1.try_into().unwrap();

			if size >= buffer.len() {
				println!("File already exists: {} ({} bytes)", filename, size);
				continue;
			}

			println!("File already exists but file size is smaller: {}", filename);
		}

		bucket.put_object(&filename, &buffer).unwrap();
		println!(
			"Successfully uploaded file: {} ({} bytes)",
			filename,
			buffer.len()
		);
	}
}

fn read_dir_recursive(dir: &path::Path) -> Vec<fs::DirEntry> {
	let mut files = Vec::new();
	if dir.is_dir() {
		for entry in fs::read_dir(dir).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_dir() {
				files.append(&mut read_dir_recursive(&path));
			} else {
				files.push(entry);
			}
		}
	}

	files
}
fn main() {
	let args = Args::parse();

	let mut paths = read_dir_recursive(&path::Path::new(&args.path));
	paths.sort_by_key(|dir| cmp::Reverse(dir.path()));

	let max_idx = cmp::min(args.max_amount, paths.len());
	upload(&args, &paths[..max_idx]);
}
