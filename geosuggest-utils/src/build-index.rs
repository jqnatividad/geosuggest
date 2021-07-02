use getopts::Options;
use std::env;

use geosuggest_core::Engine;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "geosuggest_core=info");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "output", "set output index file name", "INDEX");
    opts.optopt("c", "cities", "set geonames cities file name", "CITIES");
    opts.optopt("n", "names", "set geonames names file name", "NAMES");
    opts.optopt(
        "l",
        "languages",
        "filter names languages comma separated",
        "LANGUAGES",
    );
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f);
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    let index_file = if let Some(v) = matches.opt_str("o") {
        v
    } else {
        println!("--output option is required");
        print_usage(&program, opts);
        return Ok(());
    };

    let cities_file = if let Some(v) = matches.opt_str("c") {
        v
    } else {
        println!("--cities option is required");
        print_usage(&program, opts);
        return Ok(());
    };

    let names_file = matches.opt_str("n");

    let languages_filter = matches
        .opt_str("l")
        .map(|v| {
            v.split(',')
                .map(|i| i.trim().to_owned())
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(|| {
            if names_file.is_some() {
                panic!("Languages must be defined");
            } else {
                Vec::new()
            }
        });

    let engine = Engine::new_from_files(
        &cities_file,
        names_file.as_ref(),
        languages_filter.iter().map(AsRef::as_ref).collect(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "On build index from {} or {:?} - {}",
            &cities_file, &names_file, e
        )
    });
    engine.dump_to_json(&index_file)?;

    println!("Done. Index file: {}", &index_file);

    Ok(())
}