use std::{fmt, io, thread};
use std::process::{Child, Command, Output as ProcessOutput, Stdio};
use std::time::Duration;
use std::io::Write;

pub fn start_docker_containers(peers: u16, unls: Vec<Vec<u16>>) -> PsResult<PsOutput> {
    let unls = parse_vec_to_ps_array(unls);
    let proc_output = run_docker_script(format!("Invoke-Expression \"& C:\\Users\\Martijn.vanMeerten\\workspace\\studie\\Thesis\\ConsensusTesting\\rippled-docker\\Run.ps1 {} p {}\"", peers, unls).as_str())?;
    thread::sleep(Duration::from_millis(1000));
    let output = PsOutput::from(proc_output);
    if output.success {
        Ok(output)
    } else {
        Err(PsError::Powershell(output))
    }
}

fn parse_vec_to_ps_array(unls: Vec<Vec<u16>>) -> String {
    let mut result = "@".to_string();
    result += &format!("{:?}", unls);
    result = result.replace("[", "(");
    result = result.replace("]", ")");
    println!("UNLs: {}", result);
    result
}

fn run_docker_script(script: &str) -> PsResult<ProcessOutput> {
    let mut cmd = Command::new("pwsh");

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut process = get_process(cmd)?;

    let stdin = process.stdin.as_mut().ok_or(PsError::ChildStdinNotFound)?;
    writeln!(stdin, "{}", script)?;

    let output = process.wait_with_output()?;

    Ok(output)
}

#[cfg(target_family = "windows")]
fn get_process(mut cmd: Command) -> io::Result<Child> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    cmd.args(&["-NoProfile", "-Command", "-"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
}

#[cfg(target_family = "unix")]
fn get_process(mut cmd: Command) -> io::Result<Child> {
    cmd.args(&["-NoProfile", "-Command", "-"]).spawn()
}

type PsResult<T> = std::result::Result<T, PsError>;

#[derive(Debug, Clone)]
pub struct PsOutput {
    success: bool,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl PsOutput {
    /// Returns the parsed output of the `stdout` capture of the child process
    pub fn stdout(&self) -> Option<&str> {
        self.stdout.as_ref().map(|s| s.as_str())
    }

    /// Returns the parsed output of the `stdout` capture of the child process
    pub fn stderr(&self) -> Option<&str> {
        self.stderr.as_ref().map(|s| s.as_str())
    }
}

impl From<ProcessOutput> for PsOutput {
    fn from(proc_output: ProcessOutput) -> PsOutput {
        let stdout = if proc_output.stdout.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&proc_output.stdout).to_string())
        };

        let stderr = if proc_output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&proc_output.stderr).to_string())
        };

        PsOutput {
            success: proc_output.status.success(),
            stdout,
            stderr,
        }
    }
}

impl fmt::Display for PsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(stdout) = self.stdout().as_ref() {
            write!(f, "{}", stdout)?;
        }

        if let Some(stderr) = self.stderr().as_ref() {
            write!(f, "{}", stderr)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum PsError {
    /// An error in the PowerShell script.
    Powershell(PsOutput),
    /// An I/O error related to the child process.
    Io(io::Error),
    /// Failed to retrieve a handle to `stdin` for the child process
    ChildStdinNotFound,
}

impl std::error::Error for PsError {}

impl fmt::Display for PsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PsError::*;
        match self {
            Powershell(out) => write!(f, "{}", out)?,
            Io(e) => write!(f, "{}", e)?,
            ChildStdinNotFound => write!(f, "Failed to acquire a handle to stdin in the child process.")?,
        }
        Ok(())
    }
}

impl From<io::Error> for PsError {
    fn from(io: io::Error) -> PsError {
        PsError::Io(io)
    }
}
