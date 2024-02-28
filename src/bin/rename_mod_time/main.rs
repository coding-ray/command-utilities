use ray_file::RayFileList;

mod cli;
mod ray_file;

fn main() {
    // load command-line arguments
    let matches: clap::ArgMatches = cli::get_cli_parser().get_matches();
    let input_paths: Vec<String> = matches.get_many::<String>("input_paths").unwrap().cloned().collect();
    let time_format: String = matches.get_one::<String>("format").unwrap().clone();

    let file_list = RayFileList::from(&input_paths, time_format);
    file_list.rename_with_modification_time(true);
}
