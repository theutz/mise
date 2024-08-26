use clap::builder::StyledStr;
use console::strip_ansi_codes;
use eyre::Result;
use indoc::formatdoc;
use itertools::Itertools;

use crate::cli::Cli;
use crate::file;

/// internal command to generate markdown from help
#[derive(Debug, clap::Args)]
#[clap(hide = true)]
pub struct RenderHelp {}

impl RenderHelp {
    pub fn run(self) -> Result<()> {
        xx::file::mkdirp("docs/cli")?;
        xx::file::mkdirp("docs/.vitepress")?;
        let readme = file::read_to_string("docs/cli/index.md")?;
        let mut current_readme = readme.split("<!-- MISE:COMMANDS -->");

        let mut doc = String::new();
        doc.push_str(current_readme.next().unwrap());
        current_readme.next(); // discard existing commands
        doc.push_str(render_commands()?.as_str());
        doc.push_str(current_readme.next().unwrap());
        doc = remove_trailing_spaces(&doc) + "\n";
        file::write("docs/cli/index.md", &doc)?;
        file::write("docs/.vitepress/cli_commands.ts", render_command_ts())?;
        Ok(())
    }
}

fn render_commands() -> Result<String> {
    let mut cli = Cli::command()
        .term_width(80)
        .max_term_width(80)
        .disable_help_subcommand(true)
        .disable_help_flag(true);
    let mut doc = formatdoc!(
        r#"
            <!-- MISE:COMMANDS -->

            # Commands

    "#
    );
    for command in cli
        .get_subcommands_mut()
        .sorted_by_cached_key(|c| c.get_name().to_string())
    {
        match command.has_subcommands() {
            true => {
                let name = command.get_name().to_string();
                for subcommand in command.get_subcommands_mut() {
                    let output = render_command(Some(&name), subcommand);
                    if !subcommand.is_hide_set() {
                        doc.push_str(&output);
                    }
                    let output = output.trim().to_string() + "\n";
                    xx::file::mkdirp(format!("docs/cli/{}", name))?;
                    file::write(
                        format!("docs/cli/{}/{}.md", name, subcommand.get_name()),
                        &output,
                    )?;
                }
            }
            false => {
                let output = render_command(None, command);
                if !command.is_hide_set() {
                    doc.push_str(&output);
                }
                let output = output.trim().to_string() + "\n";
                file::write(format!("docs/cli/{}.md", command.get_name()), &output)?;
            }
        }
    }
    doc.push_str("<!-- MISE:COMMANDS -->");
    Ok(doc)
}

fn render_command(parent: Option<&str>, c: &clap::Command) -> String {
    let mut c = c.clone().disable_help_flag(true);
    let strip_usage = |s: StyledStr| {
        s.to_string()
            .strip_prefix("Usage: ")
            .unwrap_or_default()
            .to_string()
    };
    let usage = match parent {
        Some(p) => format!("{} {}", p, strip_usage(c.render_usage())),
        None => strip_usage(c.render_usage()),
    };
    let mut c = c.override_usage(&usage);

    let aliases = c.get_visible_aliases().sorted().collect_vec();
    let aliases = if !aliases.is_empty() {
        format!("\n**Aliases:** `{}`\n", aliases.join(", "))
    } else {
        String::new()
    };

    let about = strip_ansi_codes(&c.render_long_help().to_string())
        .trim()
        .to_string();
    let mut badge = String::new();
    if about.starts_with("[experimental]") {
        badge = " <Badge type=\"warning\" text=\"experimental\" />".to_string();
    }
    formatdoc!(
        "
        ## `mise {usage}`{badge}
        {aliases}
        ```text
        {about}
        ```

        ",
    )
}

fn remove_trailing_spaces(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_command_ts() -> String {
    let mut doc = String::new();
    doc.push_str(&formatdoc! {r#"
        // This file is generated by `mise render-help`
        // Do not edit this file directly

        export type Command = {{
          hide: boolean,
          subcommands?: {{
            [key: string]: Command,
          }},
        }};
        "#});
    doc.push_str("export const commands: { [key: string]: Command } = {\n");
    let mut cli = Cli::command()
        .term_width(80)
        .max_term_width(80)
        .disable_help_subcommand(true)
        .disable_help_flag(true);
    for command in cli
        .get_subcommands_mut()
        .sorted_by_cached_key(|c| c.get_name().to_string())
    {
        match command.has_subcommands() {
            true => {
                let name = command.get_name().to_string();
                doc.push_str(&format!(
                    "  \"{}\": {{\n    hide: {},\n    subcommands: {{\n",
                    name,
                    command.is_hide_set()
                ));
                for subcommand in command.get_subcommands_mut() {
                    let output = format!(
                        "      \"{}\": {{\n        hide: {},\n      }},\n",
                        subcommand.get_name(),
                        subcommand.is_hide_set()
                    );
                    doc.push_str(&output);
                }
                doc.push_str("    },\n  },\n");
            }
            false => {
                let output = format!(
                    "  \"{}\": {{\n    hide: {},\n  }},\n",
                    command.get_name(),
                    command.is_hide_set()
                );
                doc.push_str(&output);
            }
        }
    }
    doc.push_str("};\n");
    doc
}

#[cfg(test)]
mod tests {
    use std::fs;

    use indoc::indoc;
    use test_log::test;

    use crate::file;
    use crate::test::reset;

    #[test]
    fn test_render_help() {
        reset();
        file::create_dir_all("docs/cli").unwrap();
        file::write(
            "docs/cli/index.md",
            indoc! {r#"
            <!-- MISE:COMMANDS -->
            <!-- MISE:COMMANDS -->
        "#},
        )
        .unwrap();
        assert_cli!("render-help");
        let readme = fs::read_to_string("docs/cli/index.md").unwrap();
        assert!(readme.contains("# Commands"));
        file::remove_file("docs/cli/index.md").unwrap();
    }
}
