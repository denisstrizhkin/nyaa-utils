use clap::{ArgAction, Parser};
use std::error::Error;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wc", version)]
#[command(about = "word, line, and byte or character count", long_about = None)]
#[command(disable_help_flag = true, disable_version_flag = true)]
struct Args {
    /// Write to the stdout the number of characters in each input file
    #[arg(short = 'm')]
    is_count_char: bool,

    /// Write to the stdout the number of bytes in each input file
    #[arg(short = 'c')]
    is_count_byte: bool,

    /// Write to the stdout the number of <newline> characters in each input file
    #[arg(short = 'l')]
    is_count_line: bool,

    /// Write to the stdout the number of words in each input file
    #[arg(short = 'w')]
    is_count_word: bool,

    /// Input files
    files: Vec<PathBuf>,

    /// Print help
    #[arg(long, action=ArgAction::Help)]
    help: (),

    /// Print version
    #[arg(long, action=ArgAction::Version)]
    version: (),
}

#[derive(Default)]
struct Count {
    chars: Option<u64>,
    lines: Option<u64>,
    words: Option<u64>,
}

fn count<R: io::BufRead>(reader: R, args: &Args) -> Count {
    let cnt = if !(args.is_count_char
        || args.is_count_byte
        || args.is_count_line
        || args.is_count_word)
    {
        Count {
            lines: Some(0),
            chars: Some(0),
            words: Some(0),
        }
    } else {
        Count {
            lines: if args.is_count_line { Some(0) } else { None },
            chars: if args.is_count_char || args.is_count_byte {
                Some(0)
            } else {
                None
            },
            words: if args.is_count_word { Some(0) } else { None },
        }
    };
    reader.lines().flatten().fold(cnt, |cnt, line| {
        let (chars, words) = match (cnt.chars, cnt.words) {
            (None, None) => (None, None),
            (chars, words) => line.chars().fold((chars, words), |(chars, words), c| {
                (
                    chars.map(|chars| {
                        if args.is_count_char {
                            chars + 1
                        } else {
                            chars + c.len_utf8() as u64
                        }
                    }),
                    words.map(|_words| 1),
                )
            }),
        };
        Count {
            chars,
            words,
            lines: cnt.lines.map(|l| l + 1),
        }
    })
}

fn print_count(cnt: &Count, name: Option<&str>) {
    if let Some(lines) = cnt.lines {
        print!("{lines:7}");
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
        // let total = args.files
        //     .into_iter()
        //     .map(|p| fs::File::open(p))
        //     .map(io::BufReader::new);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run()?;
    Ok(())
}
