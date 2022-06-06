use std::os::unix::net::UnixStream;
use std::io::Write;
use serde::{Serialize, Deserialize};
use std::{thread, time};
use clap::Parser;

/// Wait for nscd to return the correct data
#[derive(Parser)]
struct Cli {
    #[clap(short = 's', long, default_value = PATH_NSCDSOCKET)]
    /// nscd socket to connect to
    nscd_socket: String,
    #[clap(short = 'u', long, default_value = "root")]
    /// Username to look up via nscd
    username: String,
    #[clap(short = 'i', long, default_value_t = 0)]
    /// UID to expect from the lookup
    expected_uid: u32,
    #[clap(short = 'm', long, default_value_t = 100)]
    /// Milliseconds to sleep between tries
    sleep_millis: u64,
}

/// Rust representation of struct request_header in nscd/nscd-client.h
#[repr(C, packed)]
#[derive(Serialize)]
struct RequestHeader {
    /// Version number of the daemon interface.
    version: i32,
    /// Service requested.
    request_type: i32,
    /// Key length.
    key_len: i32,
}

/// Rust representation of struct pw_response_header in nscd/nscd-client.h
#[repr(C, packed)]
#[derive(Deserialize)]
struct PwResponseHeader {
    version: i32,
    found: i32,
    pw_name_len: i32,
    pw_passwd_len: i32,
    pw_uid: u32,
    pw_gid: u32,
    pw_gecos_len: i32,
    pw_dir_len: i32,
    pw_shell_len: i32,
}

/// Rust representation of _PATH_NSCDSOCKET in nscd/nscd-client.h
const PATH_NSCDSOCKET: &str = "/var/run/nscd/socket";

/// Rust representation of NSCD_VERSION in nscd/nscd-client.h
const NSCD_VERSION: i32 = 2;

/// Rust representation of GETPWBYNAME in nscd/nscd-client.h
const GETPWBYNAME: i32 = 0;

/// Requests the lookup of a user from nscd and checks the result
fn request_user_lookup(nscd_socket: &String, username: &String, expected_uid: u32) -> std::io::Result<()> {
    let mut sock = UnixStream::connect(nscd_socket)?;

    // Send request
    let body = username.as_bytes();
    let header = RequestHeader {
        version: NSCD_VERSION,
        request_type: GETPWBYNAME,
        key_len: body.len() as i32,
    };
    match bincode::serialize_into(&sock, &header) {
        Ok(_) => {},
        Err(e) => { return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Could not serialize request: {}", e))); },
    };
    sock.write_all(body)?;

    // Read response and parse it
    let resp_header: PwResponseHeader = match bincode::deserialize_from(&sock) {
        Ok(r) => r,
        Err(e) => { return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Could not deserialize response: {}", e))); },
    };

    // Version
    let received_version = resp_header.version;
    if received_version != NSCD_VERSION {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unexpected protocol version: {}. Expected {}", received_version, NSCD_VERSION)));
    }

    // Found field
    if resp_header.found != 1 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("User {} not found", username)));
    }

    // Username
    if resp_header.pw_name_len != body.len() as i32 + 1 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Wrong name length was returned"));
    }

    // UID
    let uid = resp_header.pw_uid;
    if uid != expected_uid {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Wrong UID returned for user {}: {}", username, uid)));
    }

    Ok(())
}

fn main() {
    let args = Cli::parse();
    loop {
        match request_user_lookup(&args.nscd_socket, &args.username, args.expected_uid) {
            Ok(_) => break,
            Err(e) => println!("{}", e),
        };
        thread::sleep(time::Duration::from_millis(args.sleep_millis));
    }
}
