
extern crate getch;
extern crate clap;
extern crate quick_xml;

use clap::ArgMatches;

mod args;
mod xmltree;

mod workspace;
use workspace::{Workspace};



fn main() {
	let matches = args::get_matches();
	
	match matches.subcommand() {
		// Create a new workspace
		("new", Some(m)) => new_item(m),
		
		// Switch to a new workspace
		("switch", Some(m)) => switch_workspace(m),
		
		// Remove project or workspace
		("remove", Some(m)) => remove_item(m),
		
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


fn get_current_workspace() -> Workspace {
	Workspace::current().unwrap_or_else(|err| {
		fail_with_message(&format!("Error: {}", err));
	})
}


fn new_item(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	
	match matches.value_of("type") {
		Some("workspace") => new_workspace(name),
		Some("project") => unimplemented!(),
		Some(t) => fail_with_message(&format!("Error: {} is not recognized as internal type", t)),
		None => fail_with_message("Error: Invalid argument parameters"),
	}
	
	
}


fn new_workspace(name: &str) {
	let workspace = Workspace::new(name).unwrap_or_else(|err| {
		fail_with_message(&format!("Error: {}", err));
	});
	
	workspace.set_active();
}


fn switch_workspace(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	
	let workspace = Workspace::lookup(name).unwrap_or_else(|err| {
		fail_with_message(&format!("Error: {}", err));
	});
	
	workspace.set_active();
}


fn remove_item(matches: &ArgMatches) {
	let name = matches.value_of("name").unwrap();
	let purge = matches.is_present("purge");
	
	match matches.value_of("type") {
		Some("workspace") => Workspace::remove(name, purge).unwrap(),
		Some("project") => unimplemented!(),
		Some(t) => fail_with_message(&format!("Error: {} is not recognized as internal type", t)),
		None => fail_with_message("Error: Invalid argument parameters"),
	}
}