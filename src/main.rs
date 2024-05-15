use clap::{App, Arg};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::{env, process::Command, thread};
use std::time::{Duration, Instant};



// Function to read credentials from a file
fn read_credentials_from_file(file_path: &PathBuf) -> io::Result<(String, String)> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let lines: Vec<&str> = contents.lines().collect();
    if lines.len() >= 2 {
        println!("🪪 Reading credentials from the file: {:?}", file_path);
        Ok((lines[0].to_string(), lines[1].to_string()))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to read credentials from file"))
    }
}


// Function to write credentials to a file
fn write_credentials_to_file(file_path: &PathBuf, username: &str, password: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    writeln!(file, "{}\n{}", username, password)?;
    println!("🔏 Credentials have been written to the file: {:?}", file_path);
    Ok(())
}


fn run_applescript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("❌ Failed to execute AppleScript: {}", e))?;
    if !output.status.success() {
        Err(format!(
            "❌ AppleScript execution failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}


fn is_app_installed() -> Result<bool, String> {
    let script = r#"
    try
        tell application "Finder" to get application file id "com.cisco.anyconnect.gui"
        set appExists to true
    on error
        set appExists to false
    end try
    appExists
    "#;
    run_applescript(script).map(|output| output == "true")
}


fn ensure_app_not_running() -> Result<(), String> {
    let script = r#"
    if application "Cisco AnyConnect Secure Mobility Client" is running then
        try
            quit application "Cisco AnyConnect Secure Mobility Client"
        on error errMessage number errNumber
            do shell script "killall -9 'Cisco AnyConnect Secure Mobility Client'"
        end try
    end if
    "#;
    // Execute the script
    run_applescript(script)?;
    Ok(())
}


fn connect_vpn(username: &str, password: &str) -> Result<(), String> {
    if !is_app_installed()? {
        return Err("❌ Cisco AnyConnect Secure Mobility Client is not installed.".to_string());
    }
    ensure_app_not_running()?;
    run_applescript(r#"tell application "Cisco AnyConnect Secure Mobility Client" to activate"#)?;

    println!("⌛ Waiting for the application to launch");

    // Wait for a specific UI element to appear to ensure the app is ready
    let start = Instant::now();
    let timeout = Duration::from_secs(15); // 15 seconds timeout
    loop {
        if start.elapsed() > timeout {
            return Err("❌ Application launch timed out.".to_string());
        }

        let app_ready_script = r#"
            tell application "System Events"
                exists (window 1 of process "Cisco AnyConnect Secure Mobility Client")
            end tell
        "#;
        let app_ready: bool = run_applescript(app_ready_script)?.parse().unwrap_or(false);
        if app_ready {
            break; // Break the loop if the app is ready
        }
        thread::sleep(Duration::from_millis(500)); // Check every 500ms
    }
    run_applescript(r#"tell application "System Events" to keystroke return"#)?;

    println!("⌛ Waiting for the login screen to appear");
    // Dynamically wait for the SSO login window to appear
    let login_start = Instant::now();
    let login_timeout = Duration::from_secs(10); // 10 seconds timeout for login screen
    loop {
        if login_start.elapsed() > login_timeout {
            return Err("❌ Login screen launch timed out.".to_string());
        }
        let sso_window_script = r#"
            tell application "System Events"
                exists (window 1 of process "Cisco AnyConnect Secure Mobility Client" where title contains "Cisco AnyConnect Login")
            end tell
        "#;
        let sso_window_exists: bool = run_applescript(sso_window_script)?.parse().unwrap_or(false);
        if sso_window_exists {
            break; // Break the loop if the SSO login window is ready
        }
        thread::sleep(Duration::from_millis(500)); // Check every 500ms
    }

    // Bring SSO window to forefront if not already
    let bring_to_front_script = r#"
        tell application "System Events" to tell process "Cisco AnyConnect Secure Mobility Client"
            set frontmost to true
            perform action "AXRaise" of window 1
        end tell
    "#;
    run_applescript(bring_to_front_script)?;

    // Type the username
    let username_script = format!(r#"tell application "System Events" to keystroke "{}""#, username);
    run_applescript(&username_script)?;
    run_applescript(r#"tell application "System Events" to keystroke tab"#)?;
    thread::sleep(Duration::from_millis(500));

    // Type the password and press Enter
    let password_script = format!(r#"tell application "System Events" to keystroke "{}""#, password);
    println!("🔑 Typing the password...");
    run_applescript(&password_script)?;
    run_applescript(r#"tell application "System Events" to keystroke return"#)?;

    // Wait for the SSO window to close
    println!("⌛ Waiting for the SSO login to complete");
    let check_window_script = r#"
        tell application "System Events"
            exists (window 1 of process "Cisco AnyConnect Secure Mobility Client" where title contains "Cisco AnyConnect Login")
        end tell
    "#;
    let wait_start = Instant::now();
    let wait_timeout = Duration::from_secs(30); // Adjust the timeout as needed
    loop {
        if wait_start.elapsed() > wait_timeout {
            return Err("❌ Timeout waiting for SSO login to complete.".to_string());
        }
        let window_exists: bool = run_applescript(check_window_script)?.parse().unwrap_or(false);
        if !window_exists {
            break; // Exit the loop if the login window no longer exists
        }
        thread::sleep(Duration::from_secs(2)); // Wait for 2 seconds before checking again
    }

    Ok(())
}

// fn main() {
//     match connect_vpn("behnamm", "RAy1N!ng@*Z0fCMU") {
//         Ok(_) => println!("VPN connection initiated successfully."),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }

fn main() {
    let matches = App::new("AutoCisco")
        .version("0.1.5")
        .author("Behnam Mohammadi - https://aplaceofmind.net")
        .about("Automatically connects to CMU's Cisco VPN using credentials")
        .arg(
            Arg::with_name("stop")
                .short('s')
                .long("stop")
                .help("Stop the Cisco VPN application completely")
                .takes_value(false), // No value needed for this flag
        )
        .arg(
            Arg::with_name("username")
                .short('u')
                .long("username")
                .value_name("USERNAME")
                .help("Sets the username for VPN connection")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("password")
                .short('p')
                .long("password")
                .value_name("PASSWORD")
                .help("Sets the password for VPN connection")
                .takes_value(true),
        )
        .get_matches();

    if matches.contains_id("stop") {
        // If the stop flag is present, stop the Cisco VPN application and exit
        if let Err(e) = ensure_app_not_running() {
            eprintln!("❌ Error stopping Cisco VPN: {}", e);
        } else {
            println!("💀 Cisco VPN stopped successfully.");
        }
        return; // Exit after handling the stop command
        }
    let home_dir = env::var("HOME").expect("🏚️ Failed to find HOME directory");
    let credential_file_path = PathBuf::from(home_dir).join(".vpn_credentials");

    let (username, password) = match (matches.value_of("username"), matches.value_of("password")) {
        (Some(u), Some(p)) => {
            // Credentials provided via command-line arguments; update the credentials file
            if let Err(e) = write_credentials_to_file(&credential_file_path, u, p) {
                eprintln!("📁 Failed to write credentials to file: {}", e);
            }
            (u.to_owned(), p.to_owned())
        },
        _ => {
            // No arguments provided; try to read from the credentials file
            match read_credentials_from_file(&credential_file_path) {
                Ok((u, p)) => (u, p),
                Err(_) => {
                    eprintln!("🔐 No credentials provided and failed to read from the credentials file.");
                    return;
                },
            }
        }
    };

    // Proceed with the VPN connection using the obtained credentials
    match connect_vpn(&username, &password) {
        Ok(_) => println!("✅ VPN connection initiated successfully."),
        Err(e) => eprintln!("❌ Error connecting to VPN. Your authentication may have failed. Details: {}", e),
    }
}