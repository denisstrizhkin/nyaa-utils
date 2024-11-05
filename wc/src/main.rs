use clap::{ArgAction, Parser};
use std::error::Error;
use std::fs;
use std::io;
use std::ops::Add;
use std::path::PathBuf;

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

fn count<R: io::BufRead>(reader: R, args: &Args) -> Count {
    reader
        .lines()
        .map_while(Result::ok)
        .fold(Count::new(args), |cnt, line| {
            let chars = cnt.chars.map(|chars| {
                chars
                    + 1
                    + if args.is_char {
                        line.chars().count()
                    } else {
                        line.len()
                    }
            });
            let words = cnt.words.map(|words| {
                words
                    + line
                        .chars()
                        .fold((0, false), |(words, is_word), c| {
                            if c.is_whitespace() {
                                (words, false)
                            } else if !is_word {
                                (words + 1, true)
                            } else {
                                (words, is_word)
                            }
                        })
                        .0
            });
            Count {
                chars,
                words,
                lines: cnt.lines.map(|l| l + 1),
            }
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

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if args.files.is_empty() {
        let stdin = io::stdin().lock();
        let count = count(stdin, &args);
        print_count(&count, None);
    } else {
        let total = args
            .files
            .iter()
            .filter_map(|p| {
                let name = p.to_string_lossy();
                let file = fs::File::open(p);
                file.inspect_err(|e| eprintln!("{}: {}", name, e))
                    .ok()
                    .map(|file| {
                        let cnt = count(io::BufReader::new(file), &args);
                        print_count(&cnt, Some(name.as_ref()));
                        cnt
                    })
            })
            .fold(Count::new(&args), Count::add);
        if args.files.len() > 1 {
            print_count(&total, Some("total"));
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run()?;
    Ok(())
}
