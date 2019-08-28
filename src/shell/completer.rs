use crate::prelude::*;
use derive_new::new;
use rustyline::completion::{Completer, FilenameCompleter};

#[derive(new)]
crate struct NuCompleter {
    pub file_completer: FilenameCompleter,
    pub commands: CommandRegistry,
}

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

        let starting_quote = line_chars.get(replace_pos).map_or(None, |&v| match v {
            '"' | '\'' => Some(v),
            _ => None,
        });

        if let Some(quote_type) = starting_quote {
            for completion in &mut completions {
                if completion.replacement.contains("\\ ") {
                    completion.replacement = completion.replacement.replace("\\ ", " ");
                }
                if completion.replacement.contains("\\(") {
                    completion.replacement = completion.replacement.replace("\\(", "(");
                }

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
