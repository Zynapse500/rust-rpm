
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read, ErrorKind};


use project::{Project, ProjectList};

use serde_json;

#[derive(Clone, Serialize, Deserialize)]
pub struct Workspace {
	name: String,
	path: String
}

const WORKSPACE_PREFERENCE_FOLDER_NAME: &'static str = ".workspace";
const WORKSPACE_PROJECT_DATABASE_NAME: &'static str = "projects.json";
const WORKSPACE_PROJECTS_FOLDER_NAME: &'static str = "projects";


impl Workspace {
	
	// Creates a new workspace with the specified name
	pub fn new(name: &str, path: &str) -> Result<Workspace, String> {
		let workspace = Workspace {
			name: name.to_owned(),
			path: path.to_owned()
		};
		
		if let Err(e) = workspace.add_to_workspace_list() {
			return Err(e);
		}
		
		if let Err(_) = fs::create_dir_all(path) {
			return Err("Failed to create workspace directory!".to_owned());
		}
		
		if let Err(e) = workspace.create_preferences() {
			return Err(e);
		}
		
		Ok(workspace)
	}
	
	
	/// Add this workspace to the list of workspaces
	fn add_to_workspace_list(&self) -> Result<(), String> {
		// Deserialize list
		let mut workspaces = match WorkspaceList::get() {
			Ok(workspace) => workspace,
			Err(e) => return Err(e),
		};
		
		// Check if workspace with the same name already exists
		if let Ok(_) = workspaces.lookup_index(&self.name) {
			return Err(format!("Workspace with the name '{}' already exists", self.name).to_owned())
		}
		
		// Add the workspace to the list
		workspaces.workspaces.push(self.clone());
		
		workspaces.save()
	} 
	
	
	/// Sets this workspace as the active one
	pub fn set_active(&self) -> Result<(), String> {
		// Deserialize list
		let mut workspaces = match WorkspaceList::get() {
			Ok(workspace) => workspace,
			Err(e) => return Err(e),
		};
		
		workspaces.current = self.name.clone();
		
		workspaces.save()
	}
	
	
	/// Returns the name of this workspace
	pub fn name<'a>(&'a self) -> &'a str {
		&self.name
	}
	
	
	fn create_preferences(&self) -> Result<(), String> {
		/// Create the '.workspace' folder within the workspace root
		let mut path = PathBuf::from(&self.path);
		path.push(".workspace");
		
		if let Err(e) = fs::create_dir(&path) {
			if e.kind() != ErrorKind::AlreadyExists {
				return Err("Failed to create workspace preference folder!".to_owned());
			}
		}
		
		self.create_project_database(path)
	}
	
	
	/// Adds a project to this workspace 
	pub fn add_project(&mut self, project: Project) -> Result<(), String> {
		let mut project_list = self.get_project_list()?;
		
		if let Err(e) = project_list.add(project.clone()) {
			return Err(e);
		}
		
		let mut path = PathBuf::from(&self.path);
		path.push(WORKSPACE_PROJECTS_FOLDER_NAME);
		if let Err(e) = project.create_folder(path) {
			return Err(e);
		}
		
		project_list.save(&self.project_database_path())
	}
	
	/// Removes a project from this workspace 
	pub fn remove_project(&mut self, name: &str) -> Result<(), String> {
		let mut project_list = self.get_project_list()?;
		
		if let Err(e) = project_list.remove_project(name) {
			return Err(e);
		}
		
		project_list.save(&self.project_database_path())
	}
	
	
	/// Return the list of projects for a workspace
	pub fn get_project_list(&self) -> Result<ProjectList, String> {
		let project_database_path = self.project_database_path();
		
		// Deserialize list
		match ProjectList::get(&project_database_path) {
			Ok(project_list) => Ok(project_list),
			Err(e) => Err(e),
		}
	}
	
	
	/// Return the absolute path to a project with a name
	pub fn get_project_path(&self, name: &str) -> Result<String, String> {
		use std::path::MAIN_SEPARATOR;
		
		let project_list = self.get_project_list()?;
		
		if let Err(e) = project_list.exists(name) {
			return Err(e);
		}
		
		let mut path = PathBuf::from(&self.path);
		path.push(WORKSPACE_PROJECTS_FOLDER_NAME);
		path.push(&name.to_lowercase().replace(|c|{c == ':'}, &MAIN_SEPARATOR.to_string()));
		
		if !path.exists() {
			return Err(format!("Project folder does not exist!\nProject folder not found in: '{}' ", path.to_str().unwrap()).to_string());
		}
		
		Ok(path.to_str().unwrap().to_owned())
	}
	
	
	/// Creates the preference folder for a workspace
	fn create_project_database(&self, mut path: PathBuf) -> Result<(), String> {
		path.push(WORKSPACE_PROJECT_DATABASE_NAME);
		
		if !Path::new(&path).exists() {
			match fs::File::create(&path) {
				Ok(mut file) => match file.write_all(b"{ \"projects\": [] }") {
					Ok(_) => (),
					Err(_) => return Err("Failed to write to project databse!".to_owned())
				}
				Err(_) => return Err("Failed to create project database!".to_owned()),
			}
		}
		
		Ok(())
	}
	
	
	/// Return the path to the project preferences
	fn workspace_preferences_folder_path(&self) -> String {
		use std::path::MAIN_SEPARATOR;
		self.path.clone() + &MAIN_SEPARATOR.to_string() + WORKSPACE_PREFERENCE_FOLDER_NAME
	}
	
	/// Return the path to the project preferences
	fn project_database_path(&self) -> String {
		use std::path::MAIN_SEPARATOR;
		self.workspace_preferences_folder_path() + &MAIN_SEPARATOR.to_string() + WORKSPACE_PROJECT_DATABASE_NAME
	}
}


#[derive(Serialize, Deserialize)]
pub struct WorkspaceList {
	#[serde(default = "default_workspaces")]
	workspaces: Vec<Workspace>,
	
	#[serde(default = "default_current_workspace")]
	current: String
}

const WORKSPACES_FILE_NAME: &'static str = "workspaces.json";

fn default_current_workspace() -> String {
	"".to_owned()
}

fn default_workspaces() -> Vec<Workspace> {
	Vec::new()
}


impl WorkspaceList {
	/// Load the current workspace list from a file
	pub fn get() -> Result<WorkspaceList, String> {
		let workspace_file_path = WorkspaceList::path()?;
		
		// Add file if it doesn't exist
		if !Path::new(&workspace_file_path).exists() {
			match fs::File::create(&workspace_file_path) {
				Ok(mut file) => match file.write_all(b"{\"workspaces\": [], \"current\": \"\"}") {
					Ok(_) => (),
					Err(_) => return Err("Failed to write to workspace list file!".to_owned())
				}
				Err(_) => return Err("Failed to create workspace list file!".to_owned()),
			}
		}
		
		let mut workspace_file = String::new();
		match fs::File::open(&workspace_file_path) {
			Ok(mut file) => match file.read_to_string(&mut workspace_file) {
				Ok(_) => (),
				Err(_) => return Err("Failed to write to workspace list file!".to_owned())
			}
			Err(_) => return Err("Failed to open workspace list file!".to_owned()),
		}
		
		// Deserialize list
		let workspaces: WorkspaceList = match serde_json::from_str(&workspace_file){
			Ok(workspaces) => workspaces,
			Err(_) => return Err("Failed to load workspace list!".to_owned()),
		};
		
		Ok(workspaces)
	}
	

	/// Write the current workspace list to a file
	pub fn save(&self) -> Result<(), String> {
		let workspace_file_path = WorkspaceList::path()?;
		
		// Reseralize list
		let workspace_file = serde_json::to_string_pretty(self).unwrap();
		
		match fs::File::create(&workspace_file_path) {
			Ok(mut file) => match file.write_all(workspace_file.as_bytes()) {
				Ok(_) => (),
				Err(_) => return Err("Failed to write to workspace list file!".to_owned())
			}
			Err(_) => return Err("Failed to create/open workspace list file!".to_owned()),
		}
		
		Ok(())
	}
	
	
	// Looks up a workspace from existing workspaces
	pub fn lookup(&self, name: &str) -> Result<Workspace, String> {
		if let Some(workspace) = self.workspaces.iter().find(|elem|{ elem.name.to_lowercase() == name }) {
			return Ok(workspace.clone());
		} else {
			return Err(format!("No workspace with the name '{}'!", name).to_owned());
		}
	}
	
	
	// Looks up a workspace's index from existing workspaces
	pub fn lookup_index(&self, name: &str) -> Result<usize, String> {
		let name = name.to_lowercase();
		if let Some(index) = self.workspaces.iter().position(|elem|{ elem.name.to_lowercase() == name }) {
			return Ok(index);
		} else {
			return Err(format!("No workspace with the name '{}'!", name).to_owned());
		}
	}
	
	
	// Looks up the current workspace
	pub fn current(&self) -> Result<Workspace, String> {
		if let Ok(workspace) = self.lookup(&self.current) {
			return Ok(workspace.clone());
		} else {
			return Err("No workspace currently selected!".to_owned());
		}
	}
	
	
	/// Remves a workspace from the list, optionally removes the directory aswell
	pub fn remove(&mut self, name: &str) -> Result<(), String> {
		match self.lookup_index(name) {
			Ok(index) => {
				self.workspaces.remove(index);
			},
			Err(e) => return Err(e),
		}
		
		if self.current.to_lowercase() == name.to_lowercase() {
			self.current = "".to_owned();
		}
		
		Ok(())
	}
	
	
	/// Return the path to the workspaces file
	fn path() -> Result<String, String> {
		use std::env::current_exe;
		
		match current_exe() {
			Ok(mut exe_path) => {
				exe_path.set_file_name(WORKSPACES_FILE_NAME);
				return Ok(exe_path.to_str().unwrap().to_owned());
			}
			Err(_) => {
				return Err("Failed to get path to 'rpm' executable!".to_owned());
			}
		}
	}
}