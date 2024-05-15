use clap::{arg, command, Parser, Subcommand};
use crossterm::{
    cursor::{MoveLeft, MoveRight, MoveUp},
    execute,
    style::{Attribute, Color as TermColor, SetBackgroundColor, Stylize},
};
use kociemba::{cubie::CubieCube, facelet::FaceCube, scramble::{scramble_to_str, gen_scramble}, solver::solve as solver};
use kociemba::{error::Error, facelet::Color, scramble::scramble_from_str};
use spinners::Spinner;
use std::{
    io::{self, stdout},
    time::Instant,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "solves the cube using two-phase algorithm")]
    #[clap(group(
    clap::ArgGroup::new("state")
        .required(true)
        .args(&["scramble", "facelet"]),
    ))]
    Solve {
        #[arg(short, long)]
        scramble: Option<String>,

        #[arg(short, long)]
        facelet: Option<String>,

        #[arg(short, long, default_value_t = 20)]
        max: usize,

        #[arg(short, long, default_value_t = 3.0)]
        timeout: f32,

        #[arg(short, long)]
        verbose: bool,

        #[arg(short, long)]
        preview: bool,
    },

    #[command(about = "generates scramble")]
    Scramble {
        #[arg(short, long, default_value_t = 20)]
        length: usize,
        #[arg(short, long)]
        preview: bool,
    },
}

fn solve(
    scramble: &Option<String>,
    facelet: &Option<String>,
    max: usize,
    timeout: f32,
    verbose: bool,
    preview: bool,
) -> Result<(), Error> {
    if let Some(scramble) = scramble {
        if preview {
            let scramble = scramble_from_str(scramble)?;
            let state = CubieCube::from(&scramble);
            let facelet = FaceCube::try_from(&state)?;
            print_facelet(&facelet)?;
        }
        solve_scramble(scramble, max, timeout, verbose)?;
    } else if let Some(facelet) = facelet {
        if preview {
            let facelet = FaceCube::try_from(facelet.as_str())?;
            print_facelet(&facelet)?;
        }
        solve_facelet(facelet, max, timeout, verbose)?;
    }
    Ok(())
}

fn solve_state(cubestring: &str, max: usize, timeout: f32, verbose: bool) -> Result<(), Error> {
    let start = Instant::now();
    let mut spinner = Spinner::new(spinners::Spinners::Dots, "Solving".to_owned());
    let result = solver(&cubestring, max, timeout)?;
    let end = Instant::now();

    spinner.stop_with_newline();

    println!("Solution: {}", scramble_to_str(&result.solution)?);
    println!("Move count: {}", result.solution.len());
    println!("Solve time: {:?}", result.solve_time);
    println!("Total time: {:?}", end-start);

    Ok(())
}

fn solve_scramble(scramble: &str, max: usize, timeout: f32, verbose: bool) -> Result<(), Error> {
    let scramble = scramble_from_str(scramble)?;
    let state = CubieCube::from(&scramble);
    let fc = FaceCube::try_from(&state)?;

    solve_state(&fc.to_string(), max, timeout, verbose)
}

fn solve_facelet(facelet: &str, max: usize, timeout: f32, verbose: bool) -> Result<(), Error> {
    solve_state(facelet, max, timeout, verbose)
}

fn color_to_termcolor(color: Color) -> TermColor {
    match color {
        Color::U => TermColor::DarkYellow,
        Color::R => TermColor::Magenta,
        Color::F => TermColor::Green,
        Color::D => TermColor::White,
        Color::L => TermColor::Red,
        Color::B => TermColor::Blue,
    }
}

fn print_face(face: &[Color], offset: u16) -> Result<(), io::Error> {
    for i in 0..3 {
        let layer = format!(
            "{}  {}  {}  {}",
            SetBackgroundColor(color_to_termcolor(face[3 * i])),
            SetBackgroundColor(color_to_termcolor(face[(3 * i) + 1])),
            SetBackgroundColor(color_to_termcolor(face[(3 * i) + 2])),
            SetBackgroundColor(TermColor::Reset)
        );

        println!("{layer}");

        if offset != 0 {
            execute!(stdout(), MoveRight(offset))?;
        }
    }

    Ok(())
}

fn print_facelet(facelet: &FaceCube) -> Result<(), io::Error> {
    let stdout = stdout();

    println!();
    execute!(&stdout, MoveRight(6))?;
    print_face(&facelet.f[0..9], 6)?; // U (white)
    execute!(&stdout, MoveLeft(6))?;
    print_face(&facelet.f[36..45], 0)?; // L (orange)
    execute!(&stdout, MoveRight(6), MoveUp(3))?;
    print_face(&facelet.f[18..27], 6)?; // F (green)
    execute!(&stdout, MoveLeft(12), MoveUp(3), MoveRight(12))?;
    print_face(&facelet.f[9..18], 12)?; // R (red)
    execute!(&stdout, MoveLeft(12), MoveUp(3), MoveRight(18))?;
    print_face(&facelet.f[45..54], 18)?; // B (blue)
    execute!(&stdout, MoveLeft(12))?;
    print_face(&facelet.f[27..36], 6)?; // D (yellow)
    execute!(&stdout, MoveLeft(12))?;
    println!();

    Ok(())
}

fn scramble(length: usize, preview: bool) -> Result<(), Error> {
    let ss = gen_scramble(length)?;
    let mut cc = CubieCube::default();
    cc = cc.apply_moves(&ss);
    let fc = FaceCube::try_from(&cc)?;
    println!("Scramble: {}", scramble_to_str(&ss)?);
    if preview {
        print_facelet(&fc)?;
    }
    Ok(())
}

fn main() {
    let program = Cli::parse();

    let result = match &program.command {
        Some(Commands::Solve {
            scramble,
            facelet,
            max,
            timeout,
            verbose,
            preview,
        }) => solve(scramble, facelet, *max, *timeout, *verbose, *preview),
        Some(Commands::Scramble {
            length,
            preview,
        }) => scramble(*length, *preview),
        _ => Ok(()),
    };

    if let Err(error) = result {
        let styled = "Error:".with(TermColor::Red).attribute(Attribute::Bold);
        println!("{styled} {error}");
    }
}
