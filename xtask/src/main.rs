use std::{
    env, error,
    io::ErrorKind,
    path::PathBuf,
    process::{Command, ExitCode},
};

type Error = Box<dyn error::Error>;

fn main() -> ExitCode {
    let Some((opts, mode)) = parse() else {
        eprintln!("undefined mode");
        return ExitCode::FAILURE;
    };

    match start(opts, mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn parse() -> Option<(Opts, Mode)> {
    let mut args = env::args().skip(1);
    let mut opts = Opts { no_install: false };
    let mode = loop {
        match args.next()?.as_str() {
            "build" => break Mode::Build,
            "serve" => break Mode::Serve,
            opt => match opt.strip_prefix("--")? {
                "no-install" => opts.no_install = true,
                _ => return None,
            },
        }
    };

    Some((opts, mode))
}

struct Opts {
    no_install: bool,
}

enum Mode {
    Build,
    Serve,
}

fn start(opts: Opts, mode: Mode) -> Result<(), Error> {
    match mode {
        Mode::Build => build(opts),
        Mode::Serve => serve(opts),
    }
}

fn build(opts: Opts) -> Result<(), Error> {
    let mut cmd = Command::new("wasm-pack");
    cmd.args([
        "build",
        "web",
        "--no-pack",
        "--no-typescript",
        "--target",
        "web",
        "--out-dir",
        "../static/pkg",
    ]);

    install_and_run(&mut cmd, opts.no_install)
}

fn serve(opts: Opts) -> Result<(), Error> {
    let mut cmd = Command::new("miniserve");
    cmd.args(["--index", "index.html", "static"]);
    install_and_run(&mut cmd, opts.no_install)
}

fn install_and_run(cmd: &mut Command, no_install: bool) -> Result<(), Error> {
    add_bin_path(cmd);
    let name = cmd.get_program().to_string_lossy().into_owned();
    match run(cmd, &name) {
        Run::Ok => return Ok(()),
        Run::NotFound if no_install => return Ok(()),
        Run::NotFound => eprintln!("{name} not found, installing.."),
        Run::Failed(s) => return Err(Error::from(s)),
    }

    install(&name)?;
    match run(cmd, &name) {
        Run::Ok => Ok(()),
        Run::NotFound => Err(Error::from(format!("failed to install {name}"))),
        Run::Failed(s) => Err(Error::from(s)),
    }
}

fn add_bin_path(cmd: &mut Command) {
    let bin_path = PathBuf::from("./bin");
    let mut paths: Vec<_> = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default();

    if !paths.contains(&bin_path) {
        paths.push(bin_path);
    }

    let paths = env::join_paths(paths).expect("join paths");
    cmd.env("PATH", paths);
}

fn run(cmd: &mut Command, name: &str) -> Run {
    match cmd.status() {
        Ok(status) if status.success() => Run::Ok,
        Ok(_) => Run::Failed(format!("execution of {name} failed")),
        Err(err) if err.kind() == ErrorKind::NotFound => Run::NotFound,
        Err(err) => Run::Failed(format!("failed to run {name}: {err}")),
    }
}

enum Run {
    Ok,
    NotFound,
    Failed(String),
}

fn install(name: &str) -> Result<(), Error> {
    let status = Command::new("cargo")
        .args([
            "install",
            "--root",
            ".",
            "--target-dir",
            "target",
            "--locked",
        ])
        .arg(name)
        .status()
        .map_err(|err| format!("failed to run cargo: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::from(format!("failed to install {name}")))
    }
}
