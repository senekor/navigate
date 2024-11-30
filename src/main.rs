mod arguments;
mod config;
mod stack;

use arguments::*;
use clap::Parser;
use config::Config;
use stack::Stack;
use std::env::{current_dir, var};
use std::io::{Error, Result};
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn main() -> Result<()> {
    let args = match Arguments::try_parse() {
        Ok(a) => a,
        Err(e) => {
            print!("echo '{}'", e);
            return Ok(());
        }
    };
    let mut stack = match Stack::new(args.pid) {
        Ok(stack) => stack,
        Err(_) => {
            print!("echo ");
            return Err(Error::other("-- failed to build stack"));
        }
    };
    let res = match args.action {
        Action::push(push_args) => handle_push(&push_args, &mut stack),
        Action::pop(pop_args) => handle_pop(&pop_args, &mut stack),
        Action::stack(stack_args) => handle_stack(&stack_args, &mut stack),
        Action::bookmark(bookmark_args) => handle_bookmark(&bookmark_args, &mut stack),
    };

    if res.is_err() {
        print!("echo '{}'", res.unwrap_err());
    }
    Ok(())
}

fn handle_push(args: &PushArgs, stack: &mut Stack) -> Result<()> {
    let path = match args.path.clone() {
        Some(value) => value,
        None => {
            let home_dir = match var("HOME") {
                Ok(value) => value,
                Err(error) => return Err(Error::other(error.to_string())),
            };
            match PathBuf::from_str(&home_dir) {
                Ok(value) => value,
                Err(error) => return Err(Error::other(error.to_string())),
            }
        }
    };
    push_path(&path, stack)?;
    Ok(())
}

fn handle_pop(_args: &PopArgs, stack: &mut Stack) -> Result<()> {
    // TODO: handle arguments
    let path = stack.pop_entry()?;
    println!(
        "cd -- {}",
        match path.to_str() {
            Some(value) => value,
            None => return Err(Error::other("-- failed to print popped path as string")),
        }
    );
    Ok(())
}

fn handle_stack(args: &StackArgs, stack: &mut Stack) -> Result<()> {
    if args.stack_action.is_some() {
        match args.stack_action.clone().unwrap() {
            StackAction::clear(_) => return stack.clear_stack(),
        }
    }
    // retrieve stack
    let output = stack.get_stack()?;
    if output.is_empty() {
        return Err(Error::other("-- the stack is empty"));
    }
    // print stack to standard output
    for (n, item) in output.iter().rev().enumerate() {
        println!("echo '{} - {}'", n, item.to_str().unwrap());
    }
    Ok(())
}

fn handle_bookmark(args: &BookmarkArgs, stack: &mut Stack) -> Result<()> {
    let mut config = match Config::new() {
        Ok(value) => value,
        Err(error) => return Err(Error::other(error.to_string())),
    };
    // if args.bookmark_action.is_some() {
    if args.bookmark_action.is_some() {
        match args.bookmark_action.clone().unwrap() {
            BookmarkAction::list(_) => list_bookmarks(&mut config)?,
            BookmarkAction::add(args) => add_bookmarks(&args, &mut config)?,
            BookmarkAction::remove(args) => remove_bookmarks(&args, &mut config)?,
        };
    } else if args.name.is_some() {
        let path = match config
            .get_bookmarks()
            .get(args.name.as_ref().unwrap())
        {
            Some(value) => value,
            None => return Err(Error::other("requested bookmark does not exist")),
        };
        push_path(path, stack)?;
    } else {
        return Err(Error::other("-- provide either a `subcommand` or a `bookmark name`"));
    }
    Ok(())
}

fn list_bookmarks(config: &mut Config) -> Result<()> {
    let mut buffer = String::new();
    for (mark, path) in config.get_bookmarks() {
        buffer.push_str(&format!("{} : {}\n", mark, path.to_str().unwrap()));
    }
    println!("echo '{}'", buffer);
    Ok(())
}

fn add_bookmarks(args: &BookmarkSubArgs, config: &mut Config) -> Result<()> {
    if args.path.is_none() {
        return Err(Error::other("-- missing path argument"));
    } else {
        config.add_bookmark(&args.name, &args.path.clone().unwrap())?;
    }
    Ok(())
}

fn remove_bookmarks(args: &BookmarkSubArgs, config: &mut Config) -> Result<()> {
    config.remove_bookmark(&args.name)?;
    Ok(())
}

/// push path to stack and print command to navigate to provided path
fn push_path(path: &Path, stack: &mut Stack) -> Result<()> {
    if !path.is_dir() {
        return Err(Error::other("-- invalid path argument"));
    }
    let current_path = current_dir()?;
    let next_path = path.canonicalize()?;
    stack.push_entry(&current_path)?;
    println!(
        "cd -- {}",
        match next_path.to_str() {
            Some(value) => value,
            None => return Err(Error::other("-- failed to print provided path as string")),
        }
    );
    Ok(())
}
