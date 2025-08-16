/// Top-level declaration of modules, see:
/// https://stackoverflow.com/questions/46829539
/// https://stackoverflow.com/questions/58935890

mod constants {
    pub mod deviceinfo_constants;
    pub mod equalizer_constants;
    pub mod itunesdb_constants;
    pub mod itunesprefs_constants;
    pub mod itunessd_3g_constants;
    pub mod itunessd_constants;
    pub mod photo_database_constants;
    pub mod photofolderalbums_constants;
    pub mod playcounts_constants;
    pub mod preferences_constants;
}

mod helpers {
    pub mod helpers;
    pub mod itunesdb_helpers;
}

mod parsers {
    pub mod deviceinfo_parser;
    pub mod equalizer_parser;
    pub mod itunesdb_parser;
    pub mod itunessd_3g_parser;
    pub mod itunessd_parser;
    pub mod photo_type_parser;
    pub mod playcounts_parser;
    pub mod preferences_parser;
}

mod equalizer;
mod itunesdb;
mod itunesprefs;
mod itunessd;
mod photo_database;
mod preferences;
mod writer {
    pub mod itunesdb_writer;
}

use std::io::Read;

fn main() {
    // add a check for the number of arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        panic!(
            "Usage: {} <iTunes DB filename> <type> [format=csv|json]",
            args[0]
        );
    }

    let itunesdb_filename: String = std::env::args()
        .nth(1)
        .expect("Missing first parameter: iTunes DB filename");

    let itunesdb_file_path = std::path::Path::new(&itunesdb_filename);

    // Default to "csv" if no format specified
    let output_format = if args.len() > 3 {
        match args[3].to_lowercase().as_str() {
            "json" => "json",
            "csv" => "csv",
            "write" => "write",
            _ => {
                eprintln!("Invalid format specified. Using default 'csv'");
                "csv"
            }
        }
    } else {
        "csv"
    };

    // Early handling for writer mode: allow creating a new iTunesDB without requiring an existing file
    let itunesdb_file_type: String = std::env::args()
        .nth(2)
        .expect("Missing second parameter: iTunes DB file type");

    if itunesdb_file_type == "itunes" && output_format == "write" {
        // By default, read songs from music.json in current directory
        let songs_json_path = "music.json";
        eprintln!(
            "[writer] Writing new iTunesDB to '{}' from '{}'",
            itunesdb_filename, songs_json_path
        );
        if let Err(e) = writer::itunesdb_writer::write_itunesdb_from_json(songs_json_path, &itunesdb_filename) {
            panic!("Failed to write iTunesDB: {}", e);
        }
        return;
    }

    if !itunesdb_file_path.exists() {
        panic!(
            "No itunesDB file with that name '{}' exists",
            itunesdb_filename
        );
    }

    let itunesdb_file_length = itunesdb_file_path.metadata().unwrap().len();

    if itunesdb_file_length < 3 {
        panic!(
            "iTunesDB file '{}' has insufficient length ({})",
            itunesdb_filename, itunesdb_file_length
        );
    }

    let mut itunesdb_file_as_bytes = Vec::new();

    // https://stackoverflow.com/questions/47660946/why-does-a-file-need-to-be-mutable-to-call-readread-to-string
    let mut itunesdb_file = std::fs::File::open(itunesdb_file_path).unwrap();

    itunesdb_file
        .read_to_end(&mut itunesdb_file_as_bytes)
        .unwrap();

    // itunesdb_file_type already parsed above

    let desired_report_csv_filename = itunesdb_filename.to_string() + ".csv";

    assert!(desired_report_csv_filename != itunesdb_filename);

    if itunesdb_file_type == "photo" {
        let photos_csv_writer = helpers::helpers::init_csv_writer(&desired_report_csv_filename);
        parsers::photo_type_parser::parse_photo_type_file(
            itunesdb_file_as_bytes,
            photos_csv_writer,
        );
    } else if itunesdb_file_type == "itunes" {
        parsers::itunesdb_parser::parse_itunesdb_file(
            itunesdb_file_as_bytes,
            output_format.to_string(),
        );
    } else if itunesdb_file_type == "itprefs" {
        parsers::preferences_parser::parse_itunes_prefs_file(itunesdb_file_as_bytes);
    } else if itunesdb_file_type == "playcounts" {
        let playcounts_csv_writer = helpers::helpers::init_csv_writer(&desired_report_csv_filename);
        parsers::playcounts_parser::parse_playcounts(itunesdb_file_as_bytes, playcounts_csv_writer);
    } else if itunesdb_file_type == "pfalbums" {
        parsers::photo_type_parser::parse_photofolder_albums_file(itunesdb_file_as_bytes);
    } else if itunesdb_file_type == "preferences" {
        parsers::preferences_parser::parse_preferences_file(itunesdb_file_as_bytes);
    } else if itunesdb_file_type == "deviceinfo" {
        parsers::deviceinfo_parser::parse_device_info_file(itunesdb_file_as_bytes);
    } else if itunesdb_file_type == "equalizer" {
        let equalizer_csv_writer = helpers::helpers::init_csv_writer(&desired_report_csv_filename);
        parsers::equalizer_parser::parse_equalizer_file(
            itunesdb_file_as_bytes,
            equalizer_csv_writer,
        );
    } else if itunesdb_file_type == "shuffle" {
        print!("Parsing iTunesSD file '{}'...", itunesdb_filename);
        parsers::itunessd_parser::parse_itunessd_file(itunesdb_file_as_bytes);
    } else if itunesdb_file_type == "itunessd_3g" {
        let itunessd_3g_csv_writer =
            helpers::helpers::init_csv_writer(&desired_report_csv_filename);
        print!(
            "Parsing iTunesSD 3rd Gen Shuffle file '{}'...",
            itunesdb_filename
        );
        parsers::itunessd_3g_parser::parse_itunessd_3rdgen_file(itunesdb_file_as_bytes, itunessd_3g_csv_writer);
    } else {
        println!(
            "'{}' is not a supported iTunesDB file type!",
            itunesdb_file_type
        );
    }
}
