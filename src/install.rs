use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const EMBEDDED_CONFIG: &str = include_str!("../resources/config.example.toml");
const EMBEDDED_DESKTOP: &str = include_str!("../resources/srwc.desktop");
const EMBEDDED_PORTALS: &str = include_str!("../resources/srwc-portals.conf");

const WALLPAPER_BLUE_DRIFT: &str = include_str!("../resources/extras/wallpapers/blue_drift.glsl");
const WALLPAPER_COMPASS_GRID: &str =
    include_str!("../resources/extras/wallpapers/compass_grid.glsl");
const WALLPAPER_DARK_SEA: &str = include_str!("../resources/extras/wallpapers/dark_sea.glsl");
const WALLPAPER_DOT_GRID: &str = include_str!("../resources/extras/wallpapers/dot_grid.glsl");
const WALLPAPER_PINK_CLOUD: &str = include_str!("../resources/extras/wallpapers/pink_cloud.glsl");

pub fn run_uninstall() -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1msrwc uninstall — removing session artifacts\x1b[0m\n");

    // 1. Portals config
    if let Some(config_dir) = dirs::config_dir() {
        let portal_file = config_dir.join("xdg-desktop-portal/srwc-portals.conf");
        if portal_file.exists() {
            fs::remove_file(&portal_file)?;
            println!("  \x1b[32m✓\x1b[0m Removed {}", portal_file.display());
        } else {
            println!("  - {} not found, skipping.", portal_file.display());
        }
    }

    // 2. Wallpapers
    if let Some(data_dir) = dirs::data_local_dir() {
        let srwc_data_dir = data_dir.join("srwc");
        if srwc_data_dir.exists() {
            fs::remove_dir_all(&srwc_data_dir)?;
            println!("  \x1b[32m✓\x1b[0m Removed {}", srwc_data_dir.display());
        } else {
            println!("  - {} not found, skipping.", srwc_data_dir.display());
        }
    }

    // 3. Config directory (interactive)
    if let Some(config_dir) = dirs::config_dir() {
        let srwc_config_dir = config_dir.join("srwc");
        if srwc_config_dir.exists() {
            print!(
                "  Remove config directory {}? This will delete your config. [y/N]: ",
                srwc_config_dir.display()
            );
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                fs::remove_dir_all(&srwc_config_dir)?;
                println!("  \x1b[32m✓\x1b[0m Removed {}", srwc_config_dir.display());
            } else {
                println!("  \x1b[33m!\x1b[0m Skipped {}", srwc_config_dir.display());
            }
        }
    }

    // 4. Desktop entry (requires root)
    let desktop_path = "/usr/share/wayland-sessions/srwc.desktop";
    if PathBuf::from(desktop_path).exists() {
        println!("\n\x1b[1mSystem-level artifacts\x1b[0m");
        print!("  Remove {}? (requires sudo) [y/N]: ", desktop_path);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            let status = std::process::Command::new("sudo")
                .args(["rm", "-f", desktop_path])
                .status();

            match status {
                Ok(s) if s.success() => println!("  \x1b[32m✓\x1b[0m Removed {}", desktop_path),
                _ => {
                    println!("  \x1b[31m✗\x1b[0m Sudo removal failed.");
                    println!("  Manual command: sudo rm -f {}", desktop_path);
                }
            }
        } else {
            println!("  Skipped system-level removal.");
        }
    }

    println!("\n\x1b[32mUninstallation complete!\x1b[0m");
    Ok(())
}

pub fn run_install() -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1msrwc install — setting up session artifacts\x1b[0m\n");

    // 1. Config file
    if let Some(config_dir) = dirs::config_dir() {
        let srwc_config_dir = config_dir.join("srwc");
        fs::create_dir_all(&srwc_config_dir)?;
        let config_file = srwc_config_dir.join("config.toml");
        if config_file.exists() {
            println!(
                "  \x1b[33m!\x1b[0m {} already exists, skipping.",
                config_file.display()
            );
        } else {
            fs::write(&config_file, EMBEDDED_CONFIG)?;
            println!("  \x1b[32m✓\x1b[0m Written {}", config_file.display());
        }
    }

    // 2. Portals config
    if let Some(config_dir) = dirs::config_dir() {
        let portal_dir = config_dir.join("xdg-desktop-portal");
        fs::create_dir_all(&portal_dir)?;
        let portal_file = portal_dir.join("srwc-portals.conf");
        fs::write(&portal_file, EMBEDDED_PORTALS)?;
        println!("  \x1b[32m✓\x1b[0m Written {}", portal_file.display());
    }

    // 3. Wallpapers
    if let Some(data_dir) = dirs::data_local_dir() {
        let wallpaper_dir = data_dir.join("srwc/wallpapers");
        fs::create_dir_all(&wallpaper_dir)?;

        let wallpapers = [
            ("blue_drift.glsl", WALLPAPER_BLUE_DRIFT),
            ("compass_grid.glsl", WALLPAPER_COMPASS_GRID),
            ("dark_sea.glsl", WALLPAPER_DARK_SEA),
            ("dot_grid.glsl", WALLPAPER_DOT_GRID),
            ("pink_cloud.glsl", WALLPAPER_PINK_CLOUD),
        ];

        for (name, content) in wallpapers {
            let path = wallpaper_dir.join(name);
            fs::write(&path, content)?;
            println!("  \x1b[32m✓\x1b[0m Written {}", path.display());
        }
    }

    // 4. Desktop entry (requires root)
    println!("\n\x1b[1mSystem-level artifacts\x1b[0m");
    println!("  The .desktop file needs to go to /usr/share/wayland-sessions/ (requires root).");

    print!("  Install .desktop file now? (requires sudo) [y/N]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        let temp_desktop = std::env::temp_dir().join("srwc.desktop");
        fs::write(&temp_desktop, EMBEDDED_DESKTOP)?;

        let status = std::process::Command::new("sudo")
            .args([
                "install",
                "-Dm644",
                temp_desktop.to_str().unwrap(),
                "/usr/share/wayland-sessions/srwc.desktop",
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("  \x1b[32m✓\x1b[0m Installed /usr/share/wayland-sessions/srwc.desktop")
            }
            _ => {
                println!("  \x1b[31m✗\x1b[0m Sudo installation failed.");
                println!(
                    "  Manual command: sudo install -Dm644 <path_to_srwc.desktop> /usr/share/wayland-sessions/srwc.desktop"
                );
            }
        }
        let _ = fs::remove_file(temp_desktop);
    } else {
        println!("  Skipped system-level installation.");
        println!("  You can install it manually by running:");
        println!(
            "  echo -e \"[EMBEDDED CONTENT]\" | sudo tee /usr/share/wayland-sessions/srwc.desktop > /dev/null"
        );
    }

    println!("\n\x1b[32mInstallation complete!\x1b[0m");
    Ok(())
}
