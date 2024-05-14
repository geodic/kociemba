use clap::{arg, command, Parser, Subcommand, ValueEnum};
use crossterm::{
    cursor::{MoveLeft, MoveRight, MoveUp},
    execute,
    style::{Attribute, Color as TermColor, SetBackgroundColor, Stylize},
};
use kociemba::{cubie::CubieCube, facelet::FaceCube, solver::solve as solver, symmetries::sc};
use kociemba::{error::Error, facelet::Color, scramble::scramble_from_str};
use rand::random;
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

        #[arg(short, long, default_value_t = 23)]
        max: usize,

        #[arg(short, long)]
        timeout: f32,

        #[arg(short, long)]
        details: bool,

        #[arg(short, long)]
        preview: bool,
    },
    Tpsolver {
        #[arg(short, long)]
        facelet: Option<String>,
    },

    #[command(about = "generates scramble")]
    Scramble {
        #[arg(default_value = "random")]
        state: State,

        #[arg(short, long, default_value_t = 25)]
        number: usize,

        #[arg(short, long)]
        preview: bool,
    },
}

#[derive(ValueEnum, Clone)]
enum State {
    Random,
    // CrossSolved,
    // F2LSolved,
    // OllSolved,
    // OllCrossSolved,
    // EdgesSolved,
    // CornersSolved,
}

fn solve(
    scramble: &Option<String>,
    facelet: &Option<String>,
    max: usize,
    timeout: f32,
    details: bool,
    preview: bool,
) -> Result<(), Error> {
    if let Some(scramble) = scramble {
        if preview {
            let scramble = scramble_from_str(scramble)?;
            let state = CubieCube::from(&scramble);
            let facelet = FaceCube::try_from(&state)?;
            print_facelet(&facelet)?;
        }
        solve_scramble(scramble, max, timeout, details)?;
    } else if let Some(facelet) = facelet {
        if preview {
            let facelet = FaceCube::try_from(facelet.as_str())?;
            print_facelet(&facelet)?;
        }
        solve_facelet(facelet, max, timeout, details)?;
    }
    Ok(())
}

fn solve_state(state: CubieCube, max: usize, timeout: f32, details: bool) -> Result<(), Error> {
    let start = Instant::now();
    let mut spinner = Spinner::new(spinners::Spinners::Dots, "Solving".to_owned());
    let mut solution = solver(&state.to_string(), max, timeout);

    // let  = solve(state);
    let end = Instant::now();

    spinner.stop_with_newline();

    match solution {
        Ok(value) => {
            println!("Solution: {:?}", value);
            println!("Move count: {}", value.solution.len())
        }
        _ => println!("No solution found"),
    }

    Ok(())
}

fn solve_scramble(scramble: &str, max: usize, timeout: f32, details: bool) -> Result<(), Error> {
    let scramble = scramble_from_str(scramble)?;
    let state = CubieCube::from(&scramble);

    solve_state(state, max, timeout, details)
}

fn solve_facelet(facelet: &str, max: usize, timeout: f32, details: bool) -> Result<(), Error> {
    let solution = solver(facelet, max, timeout).unwrap();
    println!("{:?}", solution);
    Ok(())
    // if let Ok(face_cube) = FaceCube::try_from(facelet) {
    //     match CubieCube::try_from(&face_cube) {
    //         Ok(state) => Ok(solve_state(state, max, timeout, details)?),
    //         Err(_) => Err(Error::InvalidFaceletValue),
    //     }
    // } else {
    //     Err(Error::InvalidFaceletString)
    // }
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

fn tpsolver(facelet: &Option<String>) -> Result<(), Error> {
    use kociemba::solver::solver;
    let result = solver(
        "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF",
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        20,
        3.0,
    );
    println!("{:?}", result);
    Ok(())
}
fn scramble(state: &State, number: usize, preview: bool) -> Result<(), Error> {
    let mut cc = CubieCube::default();
    cc.randomize();
    let fc = FaceCube::try_from(&cc)?;
    print_facelet(&fc)?;
    // println!("{}", fc.to_string());
    let mut ss = "".to_string();
    let mut ps = 'R';
    for s in fc.to_string().chars() {
        if s != ps {
            let suffix = match random::<u16>() % 3 {
                0 => "",
                1 => "2",
                _ => "3",
            };
            ss = ss + ps.to_string().as_str() + suffix.to_string().as_str() + " ";
            // ss = ss + s.to_string().as_str() + "2 ";
        } else {
            // ss = ss + ps.to_string().as_str() + " " + s.to_string().as_str();
        }
        ps = s;
    }
    println!("{}", ss);
    let mut scramble = scramble_from_str(&ss).unwrap();
    println!("{:?}", scramble);
    let mut cc = CubieCube::default();
    scramble.truncate(25);
    cc = cc.apply_moves(&scramble);
    let fc = FaceCube::try_from(&cc).unwrap();
    if preview {
        print_facelet(&fc)?;
    }
    println!("{:?}", scramble);
    Ok(())
    // let state = match state {
    //     State::Random => {
    // State::CrossSolved => generate_state_cross_solved(),
    // State::F2LSolved => generate_state_f2l_solved(),
    // State::OllSolved => generate_state_oll_solved(),
    // State::OllCrossSolved => generate_state_oll_cross_solved(),
    // State::EdgesSolved => generate_state_edges_solved(),
    // State::CornersSolved => generate_state_corners_solved(),
    // };
    // let scramble = scramble_from_str(&state)?;
}

fn main() {
    let program = Cli::parse();

    let result = match &program.command {
        Some(Commands::Solve {
            scramble,
            facelet,
            max,
            timeout,
            details,
            preview,
        }) => solve(scramble, facelet, *max, *timeout, *details, *preview),
        Some(Commands::Scramble {
            state,
            number,
            preview,
        }) => scramble(state, *number, *preview),
        Some(Commands::Tpsolver { facelet }) => tpsolver(facelet),
        _ => Ok(()),
    };

    // if let Err(error) = result {
    //     let styled = "error:".with(TermColor::Red).attribute(Attribute::Bold);
    //     println!("{styled} {error}");
    // }
}
