use anyhow::Context;
use dialoguer::{Confirm, Input, Select};
use std::{ffi::OsString, fs, process::exit};

use crate::{
  package_manager::PackageManager,
  utils::{colors::*, theme::ColorfulTheme},
};

mod args;
mod package_manager;
mod template;
pub mod utils;

pub fn run<I, A>(args: I, bin_name: Option<String>, detected_manager: Option<String>)
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  if let Err(e) = run_cli(args, bin_name, detected_manager) {
    println!();
    eprintln!("\n {BOLD}{RED}error{RESET}: {e:#}\n");
    exit(1);
  }
}

fn run_cli<I, A>(
  args: I,
  bin_name: Option<String>,
  detected_manager: Option<String>,
) -> anyhow::Result<()>
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  handle_brand_text("\n  ⚡ Welcome To Farm ! \n");

  let detected_manager = detected_manager.and_then(|p| p.parse::<PackageManager>().ok());
  let args = args::parse(args.into_iter().map(Into::into).collect(), bin_name)?;
  let defaults = args::Args::default();
  let args::Args {
    manager,
    project_name,
    template,
  } = args;
  let cwd = std::env::current_dir()?;
  let project_name = match project_name {
    Some(name) => name,
    None => Input::<String>::with_theme(&ColorfulTheme::default())
      .with_prompt("Project name")
      .default("farm-app".into())
      .interact_text()?
      .trim()
      .into(),
  };

  let target_dir = cwd.join(&project_name);

  if target_dir.exists() && target_dir.read_dir()?.next().is_some() {
    let overwrite = Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt(format!(
        "{} directory is not empty, do you want to overwrite?",
        if target_dir == cwd {
          "Current".to_string()
        } else {
          target_dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
        }
      ))
      .default(false)
      .interact()?;
    if !overwrite {
      eprintln!("{BOLD}{RED}✘{RESET} Directory is not empty, Operation Cancelled");
      exit(1);
    }
  };

  let pkg_manager = match manager {
    Some(manager) => manager,
    None => defaults.manager.context("default manager not set")?,
  };

  let templates_no_flavors = pkg_manager.templates_no_flavors();

  let template = match template {
    Some(template) => template,
    None => {
      let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a framework:")
        .items(
          &templates_no_flavors
            .iter()
            .map(|t| t.select_text())
            .collect::<Vec<_>>(),
        )
        .default(0)
        .interact()?;

      let template = templates_no_flavors[index];
      template
      // Prompt for flavors if the template has more than one flavor
      //   let flavors = template.flavors(pkg_manager);
      //   if let Some(flavors) = flavors {
      //     let index = Select::with_theme(&ColorfulTheme::default())
      //       .with_prompt("Choose your UI flavor")
      //       .items(flavors)
      //       .default(0)
      //       .interact()?;
      //     template.from_flavor(flavors[index])
      //   } else {
      //     template
      //   }
    }
  };

  if target_dir.exists() {
    #[inline(always)]
    fn clean_dir(dir: &std::path::PathBuf) -> anyhow::Result<()> {
      for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
          if entry.file_name() != ".git" {
            clean_dir(&path)?;
            std::fs::remove_dir(path)?;
          }
        } else {
          fs::remove_file(path)?;
        }
      }
      Ok(())
    }
    clean_dir(&target_dir)?;
  } else {
    let _ = fs::create_dir_all(&target_dir);
  }

  //   Render the template
  template.render(
    &target_dir,
    pkg_manager,
    &project_name,
    &project_name,
    false,
    false,
  )?;

  handle_brand_text("\n > Initial Farm Project created successfully ✨ ✨ \n");

  if target_dir != cwd {
    handle_brand_text(&format!(
      "  cd {} \n",
      if project_name.contains(' ') {
        format!("\"{}\"", project_name)
      } else {
        project_name.to_string()
      }
    ));
  }
  if let Some(cmd) = pkg_manager.install_cmd() {
    handle_brand_text(&format!("  {} \n", cmd));
  }
  handle_brand_text(&format!("  {} \n", &pkg_manager.run_cmd()));

  Ok(())
}

fn is_valid_pkg_name(project_name: &str) -> bool {
  let mut chars = project_name.chars().peekable();
  !project_name.is_empty()
    && !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or_default()
    && !chars.any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '_') || ch.is_uppercase())
}

fn to_valid_pkg_name(project_name: &str) -> String {
  let ret = project_name
    .trim()
    .to_lowercase()
    .replace([':', ';', ' ', '~'], "-")
    .replace(['.', '\\', '/'], "");

  let ret = ret
    .chars()
    .skip_while(|ch| ch.is_ascii_digit() || *ch == '-')
    .collect::<String>();

  if ret.is_empty() {
    "farm-app".to_string()
  } else {
    ret
  }
}
