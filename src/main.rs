#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod nuhxboard;
mod ui;

use color_eyre::eyre::Context;
use gpui::{AppContext, Application};
use nuhxboard::NuhxBoard;
use std::{
    fs::{self, File},
    io::{self, prelude::*},
    path::PathBuf,
    sync::LazyLock,
};
use tracing::{debug, error, info};

static KEYBOARDS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    confy::get_configuration_file_path("NuhxBoard", None)
        .unwrap()
        .parent()
        .unwrap()
        .join("keyboards")
});

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt::init();

    if !KEYBOARDS_PATH.exists() {
        fs::create_dir_all(&*KEYBOARDS_PATH).context("Failed to create config directory")?;
    } else if !KEYBOARDS_PATH.is_dir() {
        info!("Config directory exists but is not a directory. Removing and recreating");
        fs::remove_file(&*KEYBOARDS_PATH).context("Failed to remove file at config path")?;
        fs::create_dir_all(&*KEYBOARDS_PATH).context("Failed to create config directory")?;
    }

    if fs::read_dir(&*KEYBOARDS_PATH)?.count() == 0
        && let Err(error) = fetch_sample_keyboards()
    {
        error!(?error, "Failed to get sample keyboards");
    }

    let global_path = KEYBOARDS_PATH.join("global");

    if !global_path.exists() {
        fs::create_dir_all(&global_path).context("Failed to create global theme directory")?;
    }

    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(|| {
        Application::new().run(|app| {
            app.open_window(Default::default(), |_, app| {
                app.new(|ctx| NuhxBoard::new(ctx, rx))
            })
            .unwrap();
        })
    });

    rdevin::listen(move |e| tx.send_blocking(e).unwrap()).context("Failed to start rdevin")?;

    Ok(())
}

fn fetch_sample_keyboards() -> color_eyre::Result<()> {
    info!("Downloading sample keyboards");
    let res = reqwest::blocking::get(
        "https://raw.githubusercontent.com/justdeeevin/nuhxboard/main/keyboards.zip",
    )
    .context("Failed to download sample keyboards")?;

    let mut keyboards_file = tempfile::tempfile().context("Failed to create keyboards.zip")?;

    keyboards_file
        .write_all(
            &res.bytes()
                .context("Failed to get bytes for keyboards.zip")?,
        )
        .context("Failed to write keyboards.zip")?;

    let mut keyboards_archive =
        zip::ZipArchive::new(keyboards_file).context("Failed to load keyboards.zip")?;

    info!("Extracting sample keyboards");
    let len = keyboards_archive.len();
    for i in 1..=len {
        let mut file = keyboards_archive
            .by_index(i - 1)
            .with_context(|| format!("Failed to get file #{i} from zip"))?;
        let outpath = match file.enclosed_name() {
            Some(path) => KEYBOARDS_PATH.join(path),
            None => continue,
        };
        debug!("{} ({i}/{len})", outpath.display());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)
                .with_context(|| format!("Failed to create directory {outpath:?}"))?;
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                fs::create_dir_all(p)
                    .with_context(|| format!("Failed to create directory {p:?}"))?;
            }
            let mut outfile = File::create(&outpath)
                .with_context(|| format!("Failed to create file {outpath:?}"))?;
            io::copy(&mut file, &mut outfile)
                .with_context(|| format!("Failed to populate file {outpath:?} from zip"))?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                    .with_context(|| format!("Failed to set permissions on {outpath:?}"))?;
            }
        }
    }

    Ok(())
}
