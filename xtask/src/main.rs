use crate::flags::XtaskCmd;
use xshell::{cmd, Shell};

mod flags;

fn main() -> anyhow::Result<()> {
    let flags = flags::Xtask::from_env()?;
    let sh = Shell::new()?;
    match flags.subcommand {
        XtaskCmd::Dev(_) => {
            cmd!(sh, "cargo lrun -p ehce --features bevy/dynamic_linking").run()?;
        }
        XtaskCmd::Watch(_) => {
            cmd!(sh, "cargo watch -x lcheck").run()?;
        }
        XtaskCmd::Fix(_) => {
            cmd!(sh, "cargo fmt --all").run()?;
            cmd!(sh, "cargo clippy --fix --allow-dirty --allow-staged").run()?;
            cmd!(sh, "cargo sort -w").run()?;
            cmd!(sh, "cargo-machete --fix --skip-target-dir").run()?;
        }
    }

    Ok(())
}
