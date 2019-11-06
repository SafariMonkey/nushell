use crate::prelude::*;
use derive_new::new;
use rustyline::completion::{unescape, Completer, FilenameCompleter};

#[derive(new)]
crate struct NuCompleter {
    pub file_completer: FilenameCompleter,
    pub commands: CommandRegistry,
}

const FALLBACK_QUOTE: char = '"';

const DOUBLE_QUOTES_ESCAPE_CHAR: Option<char> = Some('\\');

// Not exported from rusty_line, but needed for unescaping.
#[cfg(unix)]
const ESCAPE_CHAR: Option<char> = Some('\\');
#[cfg(windows)]
const ESCAPE_CHAR: Option<char> = None;

impl NuCompleter {
    pub fn complete(
        &self,
        line: &str,
        pos: usize,
        context: &rustyline::Context,
    ) -> rustyline::Result<(usize, Vec<rustyline::completion::Pair>)> {
        let commands: Vec<String> = self.commands.names();

        let (mut replace_pos, mut completions) =
            self.file_completer.complete(line, pos, context)?;

        let line_chars: Vec<_> = line.chars().collect();
        while replace_pos > 0 {
            if line_chars[replace_pos - 1] == ' ' {
                break;
            }
            replace_pos -= 1;
        }

        let orig_starting_quote = line_chars.get(replace_pos).map_or(None, Quote::from);

        for completion in &mut completions {
            // Remove backslashes if the completion contains '\ ' or '\(' or '\\'
            // (and we're not already completing something quoting with "'"),
            // and set the starting_quote, which will insert a double quote at
            // the beginning (but not the end) of the completion.
            // TODO: if backslashes are fully supported in paths in the future,
            // skip this step, and just pass through the backslashes.
            let starting_quote = orig_starting_quote.or_else(|| {
                if completion.replacement.contains("\\ ")
                    || completion.replacement.contains("\\(")
                    || completion.replacement.contains("\\\\")
                {
                    Some(Quote::Double)
                } else {
                    None
                }
            });
            // If the completion is to be quoted, add a quote at the start of
            // the completion. Adding a quote at the end prevents easy chain
            // completion.
            if let Some(quote_type) = starting_quote {
                completion.replacement = match orig_starting_quote {
                    Some(Quote::Single) => completion.replacement.clone(),
                    Some(Quote::Double) => {
                        unescape(&completion.replacement, DOUBLE_QUOTES_ESCAPE_CHAR).to_string()
                    }
                    None => unescape(&completion.replacement, ESCAPE_CHAR).to_string(),
                };
                if !completion.replacement.starts_with(quote_type) {
                    completion.replacement = format!("{}{}", quote_type, completion.replacement);
                }
            }
        }

        for command in commands.iter() {
            let mut pos = replace_pos;
            let mut matched = true;
            if pos < line_chars.len() {
                for chr in command.chars() {
                    if line_chars[pos] != chr {
                        matched = false;
                        break;
                    }
                    pos += 1;
                    if pos == line_chars.len() {
                        break;
                    }
                }
            }

            if matched {
                completions.push(rustyline::completion::Pair {
                    display: command.clone(),
                    replacement: command.clone(),
                });
            }
        }

        Ok((replace_pos, completions))
    }

    // fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
    //     let end = line.pos();
    //     line.replace(start..end, elected)
    // }
}

enum Quote {
    Single,
    Double,
}

impl From<&char> for Option<Quote> {
    fn from(c: &char) -> Self {
        match c {
            '"' => Some(Quote::Double),
            '\'' => Some(Quote::Single),
            _ => None,
        }
    }
}

// fn unescape(quote Quote, matches: Vec<Pair>) {

// }
