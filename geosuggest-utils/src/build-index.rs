use anyhow::Result;
#[cfg(feature = "tracing")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use geosuggest_core::{
    index::{IndexData, SourceFileOptions},
    storage, EngineData,
};
use geosuggest_utils::{IndexUpdater, IndexUpdaterSettings, SourceItem};

use clap::Parser;

/// Build index from files or urls
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Args {
    FromUrls(Urls),
    FromFiles(Files),
}

/// Build index from files
#[derive(clap::Args, Debug)]
#[command(version, about)]
struct Files {
    /// Cities file
    #[arg(long)]
    cities: String,

    /// Countries file
    #[arg(long)]
    countries: Option<String>,

    /// Names file
    #[arg(long)]
    names: Option<String>,

    /// Admin codes file
    #[arg(long)]
    admin_codes: Option<String>,

    /// Admin2 codes file
    #[arg(long)]
    admin2_codes: Option<String>,

    /// Languages
    #[arg(long)]
    languages: Option<String>,

    /// Dump index to file
    #[arg(long)]
    output: String,
}

/// Build index from urls
#[derive(clap::Args, Debug)]
#[command(version, about)]
struct Urls {
    /// Cities url
    #[arg(long)]
    cities_url: Option<String>,

    /// Citeis filename in archive
    #[arg(long)]
    cities_filename: Option<String>,

    /// Names url
    #[arg(long)]
    names_url: Option<String>,

    /// Names filename in archive
    #[arg(long)]
    names_filename: Option<String>,

    /// Countries url
    #[arg(long)]
    countries_url: Option<String>,

    /// Admin codes url
    #[arg(long)]
    admin_codes_url: Option<String>,

    /// Admin2 codes url
    #[arg(long)]
    admin2_codes_url: Option<String>,

    /// Languages
    #[arg(long)]
    languages: Option<String>,

    /// Dump index to file
    #[arg(long)]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // logging
    #[cfg(feature = "tracing")]
    {
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ))
            .with(tracing_subscriber::fmt::layer());
        subscriber.init();
    }

    match Args::parse() {
        Args::FromUrls(args) => {
            let mut settings = IndexUpdaterSettings::default();

            if let Some(url) = &args.cities_url {
                settings.cities = SourceItem {
                    url,
                    filename: args.cities_filename.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("Cities filename required to extract from archive")
                    })?,
                };
            }

            if let Some(url) = &args.names_url {
                settings.names = Some(SourceItem {
                    url,
                    filename: args.names_filename.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("Names filename required to extract from archive")
                    })?,
                });
            }

            if args.countries_url.is_some() {
                settings.countries_url = args.countries_url.as_deref();
            }

            if args.admin_codes_url.is_some() {
                settings.admin1_codes_url = args.admin_codes_url.as_deref();
            }

            if let Some(languages) = &args.languages {
                settings.filter_languages = languages.split(',').map(AsRef::as_ref).collect();
            }

            let engine = IndexUpdater::new(settings)?
                .build()
                .await
                .expect("On build index");

            storage::Storage::new()
                .dump_to(&args.output, &engine)
                .map_err(|e| anyhow::anyhow!("Failed to dump index: {e}"))?;
        }

        Args::FromFiles(args) => {
            let index_data = IndexData::new_from_files(SourceFileOptions {
                cities: args.cities,
                names: args.names,
                countries: args.countries,
                admin1_codes: args.admin_codes,
                admin2_codes: args.admin2_codes,
                filter_languages: if let Some(languages) = &args.languages {
                    languages.split(',').map(AsRef::as_ref).collect()
                } else {
                    Vec::new()
                },
            })
            .map_err(|e| anyhow::anyhow!("Failed to build index: {e}"))?;

            let engine_data = EngineData::try_from(index_data)?;

            storage::Storage::new()
                .dump_to(&args.output, &engine_data)
                .map_err(|e| anyhow::anyhow!("Failed to dump index: {e}"))?;
        }
    };

    Ok(())
}
