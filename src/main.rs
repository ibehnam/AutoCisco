use clap::{App, Arg};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::{env, process::Command, thread, time::Duration};


// Function to read credentials from a file
fn read_credentials_from_file(file_path: &PathBuf) -> io::Result<(String, String)> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let lines: Vec<&str> = contents.lines().collect();
    if lines.len() >= 2 {
        Ok((lines[0].to_string(), lines[1].to_string()))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to read credentials from file"))
    }
}


// Function to write credentials to a file
fn write_credentials_to_file(file_path: &PathBuf, username: &str, password: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    writeln!(file, "{}\n{}", username, password)?;
    Ok(())
}


fn run_applescript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

    if !output.status.success() {
        Err(format!(
            "AppleScript execution failed with error: {}",
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
    // Print a message to the console
    println!("The app was already running and has been restarted.");
    Ok(())
}


fn connect_vpn(username: &str, password: &str) -> Result<(), String> {
    if !is_app_installed()? {
        return Err("Cisco AnyConnect Secure Mobility Client is not installed.".to_string());
    }
    ensure_app_not_running()?;
    // Open Cisco AnyConnect
    run_applescript(r#"tell application "Cisco AnyConnect Secure Mobility Client" to activate"#)?;
    // Wait for the application to launch and the window to become active
    thread::sleep(Duration::from_secs(2));
    // Press Enter to proceed to the login screen
    run_applescript(r#"tell application "System Events" to keystroke return"#)?;
    // Wait for the login screen to appear
    thread::sleep(Duration::from_secs(2));
    // Type the username
    let username_script = format!(r#"tell application "System Events" to keystroke "{}""#, username);
    run_applescript(&username_script)?;
    run_applescript(r#"tell application "System Events" to keystroke tab"#)?;
    // Type the password
    let password_script = format!(r#"tell application "System Events" to keystroke "{}""#, password);
    run_applescript(&password_script)?;
    run_applescript(r#"tell application "System Events" to keystroke return"#)?;
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
        .version("0.1.2")
        .author("Behnam Mohammadi - https://aplaceofmind.net")
        .about("Automatically connects to CMU's Cisco VPN using credentials")
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

    let home_dir = env::var("HOME").expect("Failed to find HOME directory");
    let credential_file_path = PathBuf::from(home_dir).join(".vpn_credentials");

    let (username, password) = match (matches.value_of("username"), matches.value_of("password")) {
        (Some(u), Some(p)) => {
            // Credentials provided via command-line arguments; update the credentials file
            if let Err(e) = write_credentials_to_file(&credential_file_path, u, p) {
                eprintln!("Failed to write credentials to file: {}", e);
            }
            (u.to_owned(), p.to_owned())
        },
        _ => {
            // No arguments provided; try to read from the credentials file
            match read_credentials_from_file(&credential_file_path) {
                Ok((u, p)) => (u, p),
                Err(_) => {
                    eprintln!("No credentials provided and failed to read from the credentials file.");
                    return;
                },
            }
        }
    };

    // Proceed with the VPN connection using the obtained credentials
    match connect_vpn(&username, &password) {
        Ok(_) => println!("VPN connection initiated successfully."),
        Err(e) => eprintln!("Error connecting to VPN: {}", e),
    }
}