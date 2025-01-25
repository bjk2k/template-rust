mod application;
use clap::{arg, command, Command};
use color_eyre::eyre::eyre;
use keyring::Entry;

/// Given an API key, store it in the keychain.
///
/// # Errors
///
/// This function will return an error if the key is not the correct format.
/// Also if the API key couldn't be stored in the keychain.
fn register_api_key(api_key: &str) -> color_eyre::Result<()> {
    println!("Registering API key: {}", api_key);

    let entry = Entry::new("monika-cli", "api_key")?;

    let api_key_expected_len = application::API_KEY_LEN;

    if api_key.len() == api_key_expected_len {
        entry.set_password(api_key)?;
        println!("API key successfully stored.");
    } else {
        return Err(eyre!(
            "API key stored, but it is the wrong length. \
            Expected {} bytes, found {} bytes.",
            api_key_expected_len,
            api_key.len()
        ));
    }

    Ok(())
}

/// Validate the configuration and run the application loop.
///
/// # Errors
///
/// This function will return an error if the API key is not found or if it is
/// the wrong length. Also if the application loop panics.
fn validate_config_and_run() -> color_eyre::Result<()> {
    let entry = Entry::new("monika-cli", "api_key")?;
    let api_key = entry.get_password();

    match api_key {
        Ok(api_key) => {
            if api_key.len() == application::API_KEY_LEN {
                let api_key_bytes: [u8; application::API_KEY_LEN] =
                    api_key.as_bytes().try_into()?;

                application::application_loop(api_key_bytes)
            } else {
                // return
                Err(eyre!(
                    "API key found, but it is the wrong length. \
                    Expected {} bytes, found {} bytes.",
                    application::API_KEY_LEN,
                    api_key.len()
                ))
            }
        }
        Err(_) => {
            // return
            Err(eyre!(
                "No API key found. Please run `monika login` to store an API key."
            ))
        }
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let matches = command!() // requires `cargo` feature
        .propagate_version(true)
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("login")
                .about("Parse an API to store in the keychain.")
                .arg(
                    arg!([API_KEY])
                        .help(
                            "API key to store in the keychain. \
                    This is the key used to authenticate with the API.",
                        )
                        .required(true),
                ),
        )
        .subcommand(Command::new("run").about("Run the application."))
        .get_matches();

    match matches.subcommand() {
        Some(("login", sub_matches)) => register_api_key(
            sub_matches
                .get_one::<String>("API_KEY")
                .expect("API_KEY is required"),
        ),
        _ => validate_config_and_run(),
    }
}
