
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


macro_rules! try_fatal {
	($e:expr) => (
		($e).unwrap_or_else(|err|{fail_with_error(err)})
	)
}


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
	try_fatal!(try_fatal!(WorkspaceList::get()).current())
}


fn get_confirmation(message: &str) -> bool {
	use std::io::{stdin, stdout, Write};
	
	print!("{} (Y/n) ", message);
	stdout().flush();
	
	let mut answer = String::new();
	
	stdin().read_line(&mut answer).expect("Did not enter a correct string");
	
	if let Some('\n')= answer.chars().next_back() {
		answer.pop();
	}
	if let Some('\r')= answer.chars().next_back() {
		answer.pop();
	}
	
	return if "y" == answer.to_lowercase() {
		true
	} else {
		false
	}
}


fn new_item(matches: &ArgMatches) {
	use std::env::current_dir;
	use std::path::MAIN_SEPARATOR;
	
	let name = matches.value_of("name").unwrap();
	let path = {
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
	let workspace = try_fatal!(Workspace::new(name, path));
	try_fatal!(workspace.set_active());
}


fn new_project(name: &str) {
	let mut workspace = get_current_workspace();
	try_fatal!(workspace.add_project(Project::from_str(name)));
}


fn switch_workspace(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	let workspace_list = try_fatal!(WorkspaceList::get());
	let workspace = try_fatal!(workspace_list.lookup(name));
	try_fatal!(workspace.set_active());
}


fn display_current_workspace(matches: &ArgMatches) {
	let workspace_list = try_fatal!(WorkspaceList::get());
	let current = try_fatal!(workspace_list.current());
	
	if matches.is_present("list projects") {
		let project_list = try_fatal!(current.get_project_list());
		
		println!("{}", project_list);
	} else {
		println!("Current workspace: '{}'", current.name());
	}
}


fn remove_item(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	
	match matches.value_of("type") {
		Some("workspace") =>remove_workspace(name),
		Some("project") => remove_project(name), // fail_with_message("Removal of projects has been disabled in order to prevent unwanted loss of projects/data"),
		Some(t) => fail_with_message(&format!("Error: '{}' is not recognized as internal type", t)),
		None => fail_with_message("Error: Invalid argument parameters"),
	}
}


fn remove_workspace(name: &str) {
	let mut workspace_list = try_fatal!(WorkspaceList::get());
	try_fatal!(workspace_list.remove(name));
	try_fatal!(workspace_list.save());
}


fn remove_project(name: &str) {
	if get_confirmation(&format!("Are you sure you want to remove the project '{}'? This is an irreversible action!", name)) {
		let mut current_workspace = get_current_workspace();
		try_fatal!(current_workspace.remove_project(name));
		println!("Project removed!");
	} else {
		println!("Removal of project aborted!");
	}
}


fn open_project(matches: &ArgMatches) {
	use std::process::Command;
	
	let name = matches.value_of("name").unwrap();
	
	let current_workspace = get_current_workspace();
	
	let project_path = try_fatal!(current_workspace.get_project_path(name));
	
	if cfg!(target_os = "windows") {
		if let Err(e) = Command::new("explorer")
			.arg(&project_path)
			.spawn() {
			fail_with_message(&format!("Command Error: {}", e));
		}
	}
}
