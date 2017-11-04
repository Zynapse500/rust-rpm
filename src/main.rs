
extern crate getch;
extern crate clap;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use clap::ArgMatches;

mod args;

mod workspace;
use workspace::{Workspace, WorkspaceList};

mod project;
use project::Project;


use std::path::PathBuf;


fn main() {
	let matches = args::get_matches();
	
	match matches.subcommand() {
		// Create a new workspace
		("new", Some(m)) => new_item(m),
		
		// Remove project or workspace
		("remove", Some(m)) => remove_item(m),
		
		// Switch to a new workspace
		("switch", Some(m)) => switch_workspace(m),
		
		// Open a project
		("open", Some(m)) => open_project(m),
		
		// Display the current workspace
		("current", Some(m)) => display_current_workspace(m),
		
		_ => ()
	}
	
	pause();
}

fn pause() {
	let getch = getch::Getch::new();
	
	println!("Press any key to continue...");
	getch.getch().unwrap();
}


fn fail_with_message(msg: &str) -> ! {
	println!("{}", msg);
	pause();
	std::process::exit(1);
}


fn fail_with_error(err: String) -> ! {
	fail_with_message(&format!("Error: {}", err));
}


fn get_current_workspace() -> Workspace {
	let workspace_list = WorkspaceList::get().unwrap_or_else(|err|{fail_with_error(err)});
	
	workspace_list.current().unwrap_or_else(|err| {
		fail_with_message(&format!("Error: {}", err));
	})
}


fn new_item(matches: &ArgMatches) {
	use std::env::current_dir;
	use std::path::MAIN_SEPARATOR;
	
	let name = matches.value_of("name").unwrap();
	let mut path = {
		let mut absolute_path = current_dir().unwrap();
		absolute_path.push( PathBuf::from(
		if matches.is_present("path") {
			matches.value_of("path").unwrap().to_owned() + &MAIN_SEPARATOR.to_string() + name
		} else {
			name.to_owned()
		}));
		absolute_path.to_str().unwrap().to_owned()
	};
	
	
	match matches.value_of("type") {
		Some("workspace") => new_workspace(name, &path),
		Some("project") => new_project(name),
		Some(t) => fail_with_message(&format!("Error: {} is not recognized as internal type", t)),
		None => fail_with_message("Error: Invalid argument parameters"),
	}
}


fn new_workspace(name: &str, path: &str) {
	let workspace = Workspace::new(name, path).unwrap_or_else(|err|{fail_with_error(err)});
	
	workspace.set_active().unwrap_or_else(|err|{fail_with_error(err)});
}


fn new_project(name: &str) {
	let mut workspace = get_current_workspace();
	workspace.add_project(Project::from_str(name)).unwrap_or_else(|err|{fail_with_error(err)});
}


fn switch_workspace(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	
	let workspace_list = WorkspaceList::get().unwrap_or_else(|err|{fail_with_error(err)});
	
	let workspace = workspace_list.lookup(name).unwrap_or_else(|err|{fail_with_error(err)});
	
	workspace.set_active().unwrap_or_else(|err|{fail_with_error(err)});
}


fn display_current_workspace(matches: &ArgMatches) {
	let workspace_list = WorkspaceList::get().unwrap_or_else(|err|{fail_with_error(err)});
	let current = workspace_list.current().unwrap_or_else(|err|{fail_with_error(err)});
	
	println!("Current workspace: '{}'", current.name());
}


fn remove_item(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	let purge = matches.is_present("purge");
	
	match matches.value_of("type") {
		Some("workspace") =>remove_workspace(name, purge),
		Some("project") => unimplemented!(),
		Some(t) => fail_with_message(&format!("Error: '{}' is not recognized as internal type", t)),
		None => fail_with_message("Error: Invalid argument parameters"),
	}
}



fn remove_workspace(name: &str, purge: bool) {
	let mut workspace_list = WorkspaceList::get().unwrap_or_else(|err|{fail_with_error(err)});
	
	workspace_list.remove(name, purge).unwrap_or_else(|err|{fail_with_error(err)});
	
	workspace_list.save().unwrap_or_else(|err|{fail_with_error(err)});
}


fn open_project(matches: &ArgMatches) {
	use std::process::Command;
	
	let name = matches.value_of("name").unwrap();
	
	let current_workspace = get_current_workspace();
	
	let (project, path) = current_workspace.lookup_project_with_path(name).unwrap_or_else(|err|{fail_with_error(err)});
	
	if cfg!(target_os = "windows") {
		Command::new("explorer")
			.arg(&path)
			.spawn();
	}
}