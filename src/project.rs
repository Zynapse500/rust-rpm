
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Write, Read, ErrorKind};

use serde_json;

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
	name: String,
	projects: Vec<Project>
}


impl Project {
	/// Create a new project, with subprojects in the format 'project1:project2:project3' etc
	pub fn from_str(text: &str) -> Self {
		let names: Vec<&str> = text.splitn(2, ":").collect();
		
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
	
	
	/// Searches the project list for a project with the correct name
	pub fn lookup_project_with_path(&self, name: &str) -> Result<(Project, String), String> {
		//let candidates = Vec::new();
		
		unimplemented!()
	}
}