
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Write, Read, ErrorKind};

use std::fmt;

use serde_json;


#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
	name: String,
	projects: Vec<Project>
}


impl Project {
	/// Create a new project, with subprojects in the format 'project1:project2:project3' etc
	pub fn from_str(text: &str) -> Self {
		let names: Vec<&str> = text.splitn(2, |c|{ c == ':' || c == '/' || c == '\\'}).collect();
		
		let mut projects = Vec::new();
		if names.len() > 1 {
			projects.push(Project::from_str(names[1]));
		}
		
		Project {
			name: names[0].to_owned(),
			projects
		}
	}
	
	
	/// Creates the project's folder
	pub fn create_folder(&self, mut path: PathBuf) -> Result<(), String> {
		path.push(&self.name);
		if let Err(e) = fs::create_dir_all(path.clone()) {
			if e.kind() != ErrorKind::AlreadyExists {
				return Err("Failed to create project directory!".to_owned());
			}
		}
		
		for project in self.projects.iter() {
			if let Err(e) = project.create_folder(path.clone()) {
				return Err(e);
			}
		}
		
		Ok(())
	}
	
	
	/// Adds a new subproject to this project
	pub fn add(&mut self, project: Project) -> Result<(), String> {
		let name = project.name.to_lowercase();
		for proj in self.projects.iter_mut() {
			if proj.name.to_lowercase() == name {
				if project.projects.len() > 0 {
					for p in project.projects {
						proj.add(p);
					}
					return Ok(());
				} else {
					return Err("Project with that name already exists!".to_owned());
				}
			}
		}
		
		self.projects.push(project);
		Ok(())
	}
	
	
	/// Remove a project from the project list
	pub fn remove(&mut self, name: &str) -> Result<(), String> {
		let name = name.to_lowercase().replace(|c|{c == '/' || c == '\\'}, ":");
		let project_names: Vec<&str> = name.splitn(2, ':').collect();
		
		let mut remove_index = None;
		for (index, project) in self.projects.iter_mut().enumerate() {
			if project.name == project_names[0] {
				if project_names.len() > 1 {
					return project.remove(project_names[1]);
				} else {
					remove_index = Some(index);
				}
			}
		}
		
		if let Some(index) = remove_index {
			self.projects.remove(index);
		} else {
			return Err(format!("Failed to find project with matching name!"));
		}
		Ok(())
	}
}


#[derive(Serialize, Deserialize)]
pub struct ProjectList {
	#[serde(default = "default_projects")]
	projects: Vec<Project>
}

fn default_projects() -> Vec<Project> {
	Vec::new()
}


impl ProjectList {
	/// Load the current workspace list from a file
	pub fn get(path: &str) -> Result<ProjectList, String> {
		// Add file if it doesn't exist
		if !Path::new(path).exists() {
			return Err("Project database doesn't exist!".to_owned());
		}
		
		let mut project_file = String::new();
		match fs::File::open(path) {
			Ok(mut file) => match file.read_to_string(&mut project_file) {
				Ok(_) => (),
				Err(_) => return Err("Failed to write to project database!".to_owned())
			}
			Err(_) => return Err("Failed to open project database!".to_owned()),
		}
		
		// Deserialize list
		let project_list: ProjectList = match serde_json::from_str(&project_file){
			Ok(project_list) => project_list,
			Err(_) => return Err("Failed to load project database!".to_owned()),
		};
		
		Ok(project_list)
	}
	

	/// Write the current project list to a file
	pub fn save(&self, path: &str) -> Result<(), String> {
		// Reseralize list
		let project_list = serde_json::to_string_pretty(self).unwrap();
		
		match fs::File::create(&path) {
			Ok(mut file) => match file.write_all(project_list.as_bytes()) {
				Ok(_) => (),
				Err(_) => return Err("Failed to write to project database!".to_owned())
			}
			Err(_) => return Err("Failed to create/open project database!".to_owned()),
		}
		
		Ok(())
	}
	
	
	/// Add a project to the project list
	pub fn add(&mut self, project: Project) -> Result<(), String> {
		let name = project.name.to_lowercase();
		for proj in self.projects.iter_mut() {
			if proj.name.to_lowercase() == name {
				if project.projects.len() > 0 {
					for p in project.projects {
						proj.add(p)?;
					}
					return Ok(());
				} else {
					return Err("Project with that name already exists!".to_owned());
				}
			}
		}
		
		self.projects.push(project);
		Ok(())
	}
	
	
	/// Remove a project from the project list
	pub fn remove_project(&mut self, name: &str) -> Result<(), String> {
		if let Err(e) = self.exists(name) {
			return Err(e);
		}
		
		let name = name.to_lowercase().replace(|c|{c == '/' || c == '\\'}, ":");
		let project_names: Vec<&str> = name.splitn(2, ':').collect();
		
		let mut remove_index = None;
		for (index, project) in self.projects.iter_mut().enumerate() {
			if project.name == project_names[0] {
				if project_names.len() > 1 {
					return project.remove(project_names[1]);
				} else {
					remove_index = Some(index);
				}
			}
		}
		
		if let Some(index) = remove_index {
			self.projects.remove(index);
		} else {
			return Err(format!("Failed to find project with matching name!"));
		}
		
		Ok(())
	}

	
	/// Returns all the names in the list, recursively separated by colons: ["project1", "project2", "project1:project11"]
	pub fn get_project_names(&self) -> Vec<String> {
		use std::collections::VecDeque;
		
		let mut names = Vec::new();
		let mut projects: VecDeque<(&Project, Option<String>)> = VecDeque::new();
		
		for project in self.projects.iter() {
			projects.push_back((project, None));
		}
		
		while let Some((project, root)) = projects.pop_back() {
			let name = match root {
				Some(root) => root.clone() + ":" + &project.name,
				None => project.name.clone()
			};
			
			names.push(name.clone());
			
			for proj in project.projects.iter() {
				projects.push_back((proj, Some(name.clone())));
			}
		}
		
		names
	}
	
	
	/// Returns true if a project with a name exists
	pub fn exists(&self, name: &str) -> Result<(), String> {
		let name = name.to_lowercase().replace(|c|{c == '/' || c == '\\'}, ":");
		let names: Vec<String> = self.get_project_names().into_iter().map(|string|{string.to_lowercase()}).collect();
		
		for proj_name in names.iter() {
			if *proj_name == name {
				return Ok(());
			}
		}
		
		// No match was found, search for close matches
		let mut matches = Vec::new();
		
		for proj_name in names.iter() {
			if proj_name.contains(&name) || name.contains(proj_name) {
				matches.push(proj_name);
			}
		}
		
		let mut text = String::from(format!("No projects with the name '{}' found!", name));
		if matches.len() > 0 {
			text += "\nDid you mean:";
			
			for proposal in matches.iter() {
				text += "\n";
				text += proposal;
			}
		}
		Err(text)
	}
}


impl fmt::Display for ProjectList {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::collections::HashSet;
		
		let mut text = String::from("Projects\n");
		
		{
			fn add_project(mut text: String, project: &Project, mut bar_levels: HashSet<usize>, depth: usize, last: bool) -> String {
				
				let mut vec = Vec::new();
				for elem in bar_levels.iter() {
					vec.push(elem.clone());
				}
				
				for i in 0..depth {
					text += if i + 1 == depth {
						if last {
							bar_levels.remove(&i);
							"└───"
						} else {
							"├───"
						}
					} else if bar_levels.contains(&i) {
						"│   "
					} else {
						"    "
					}
				}
				
				text += &project.name;
				text += "\n";
				
				bar_levels.insert(depth);
				
				for (index, proj) in project.projects.iter().enumerate() {
					text = add_project(text, proj, bar_levels.clone(), depth + 1, index == project.projects.len() - 1);
				}
				
				text
			};
			
			for (index, project) in self.projects.iter().enumerate() {
				text = add_project(text, project, {let mut set = HashSet::new(); set.insert(0); set}, 1, index == self.projects.len() - 1);
			}
		}
		
		
		write!(f, "{}", text)
	}
}
