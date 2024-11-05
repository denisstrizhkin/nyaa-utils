use clap::{ArgAction, Parser};
use std::error::Error;
use std::fs;
use std::io;
use std::ops::Add;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version)]
#[command(about = "word, line, and byte or character count", long_about = None)]
#[command(disable_help_flag = true, disable_version_flag = true)]
struct Args {
    /// Write to the stdout the number of characters in each input file
    #[arg(short = 'm')]
    is_char: bool,

    /// Write to the stdout the number of bytes in each input file
    #[arg(short = 'c')]
    is_byte: bool,

    /// Write to the stdout the number of <newline> characters in each input file
    #[arg(short = 'l')]
    is_line: bool,

    /// Write to the stdout the number of words in each input file
    #[arg(short = 'w')]
    is_word: bool,

    /// Input files
    files: Vec<PathBuf>,

    /// Print help
    #[arg(long, action=ArgAction::Help)]
    help: (),

    /// Print version
    #[arg(long, action=ArgAction::Version)]
    version: (),
}

#[derive(Clone, Copy)]
struct Count {
    chars: Option<usize>,
    lines: Option<usize>,
    words: Option<usize>,
}

impl Count {
    fn new(args: &Args) -> Self {
        if !(args.is_char || args.is_byte || args.is_line || args.is_word) {
            Count {
                lines: Some(0),
                chars: Some(0),
                words: Some(0),
            }
        } else {
            Count {
                lines: if args.is_line { Some(0) } else { None },
                chars: if args.is_char || args.is_byte {
                    Some(0)
                } else {
                    None
                },
                words: if args.is_word { Some(0) } else { None },
            }
        }
    }
}

impl Add for Count {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            chars: self.chars.zip(other.chars).map(|(a, b)| a + b),
            words: self.words.zip(other.words).map(|(a, b)| a + b),
            lines: self.lines.zip(other.lines).map(|(a, b)| a + b),
        }
    }
}

fn count<R: io::BufRead>(reader: R, init_cnt: &Count, is_char: bool) -> Result<Count, io::Error> {
    reader.lines().try_fold(*init_cnt, |cnt, line| match line {
        Ok(line) => {
            let chars = cnt.chars.map(|chars| {
                chars
                    + 1
                    + if is_char {
                        line.chars().count()
                    } else {
                        line.len()
                    }
            });
            let words = cnt
                .words
                .map(|words| words + line.split_whitespace().count());
            Ok(Count {
                chars,
                words,
                lines: cnt.lines.map(|l| l + 1),
            })
        }
        Err(e) => Err(e),
    })
}

fn print_count(cnt: &Count, name: Option<&str>) {
    if let Some(lines) = cnt.lines {
        print!(" {lines:7}");
    }
    if let Some(words) = cnt.words {
        print!(" {words:7}");
    }
    if let Some(chars) = cnt.chars {
        print!(" {chars:7}");
    }
    if let Some(name) = name {
        print!(" {name}");
    }
    println!();
}

enum Input<'a> {
    Stdin(io::Stdin),
    File(&'a Path),
}

fn process_inputs(inputs: &[Input], args: &Args) {
    let cnt = Count::new(args);
    let total = inputs
        .iter()
        .filter_map(|input| {
            let (cnt, name) = match input {
                Input::Stdin(stdin) => (count(stdin.lock(), &cnt, args.is_char), None),
                Input::File(path) => (
                    fs::File::open(path)
                        .and_then(|file| count(io::BufReader::new(file), &cnt, args.is_char)),
                    Some(path.to_string_lossy()),
                ),
            };
            cnt.inspect_err(|e| {
                name.as_ref().inspect(|name| eprint!("{name}: "));
                eprintln!("{e}");
            })
            .ok()
            .inspect(|cnt| print_count(cnt, name.as_deref()))
        })
        .fold(cnt, Count::add);
    if inputs.len() > 1 {
        print_count(&total, Some("total"));
    }
}

fn run() {
    let args = Args::parse();
    let inputs: Vec<Input> = if args.files.is_empty() {
        vec![Input::Stdin(io::stdin())]
    } else {
        args.files.iter().map(|p| Input::File(p)).collect()
    };
    process_inputs(&inputs, &args);
}

fn main() -> Result<(), Box<dyn Error>> {
    run();
    Ok(())
}
