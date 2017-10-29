
use std::fs;
use std::io::ErrorKind;

use xmltree::{XmlTree, XmlElement};
use project::Project;


pub struct Workspace {
	pub name: String,
	pub path: String
}

const DEFAULT_FILE: &'static str = 
"
<workspaces>
</workspaces>
";

const WORKSPACES_FILE_PATH: &'static str = "workspaces.xml";

impl Workspace {
	
	// Creates a new workspace with the specified name
	pub fn new(path: &str) -> Result<Workspace, String> {
		use std::path::{MAIN_SEPARATOR};
		let path = if MAIN_SEPARATOR == '\\' {path.replace("/", "\\")} else {path.replace("\\", "/")};
		
		// Get the name of the workspace
		let name = (&path.chars().rev().take_while(|c| { *c != '/' && *c != '\\' }).collect::<String>()).chars().rev().collect::<String>();
		
		// Check if workspace already exists
		if let Ok(_) = Workspace::lookup(&name) {
			return Err("Workspace with that name already exists".to_owned());
		}
		
		// Create the workspace folder
		if let Err(e) = fs::create_dir_all(&path) {
			return Err(format!("Failed to create workspace: {}", e));
		}
		
		// Get the path to the workspace folder
		use std::env::current_dir;
		let workspace_path = current_dir().unwrap().join(&path).to_str().unwrap().to_owned();
		
		let workspace = Workspace {
			name,
			path: workspace_path
		};
		
		// Create preference folder
		if let Err(e) = workspace.setup_preferences() {
			return Err(format!("Failed to create workspace: {}", e));
		}
		
		
		if let Err(e) = workspace.save() {
			match e {
				ErrorKind::AlreadyExists => return Err("Workspace with that name already exists".to_owned()),
				_ => panic!("Failed to save workspace")
			}
		}
		
		
		Ok(workspace)
	}
	
	
	// Looks up a workspace from existing workspaces
	pub fn lookup(name: &str) -> Result<Workspace, String> {
		let tree = XmlTree::from_file(WORKSPACES_FILE_PATH);
		
		for root in tree.roots() {
			if root.tag() == "workspaces" {
				for elem in root.elements() {
					if let Some(workspace) = Workspace::from_xml_element(elem) {
						if workspace.name == name {
							return Ok(workspace);
						}
					}
				}
			}
		}
		
		Err(format!("Error: \"Could not find workspace with the name '{}'\"", name))
	}
	
	
	// Looks up the current workspace
	pub fn current() -> Result<Workspace, String> {
		let tree = XmlTree::from_file(WORKSPACES_FILE_PATH);
		
		for root in tree.roots() {
			if root.tag() == "workspaces" {
				for elem in root.elements() {
					if elem.tag() == "current" {
						for attrib in elem.attributes() {
							if attrib.key == "name" {
								return Workspace::lookup(&attrib.value);
							}
						}
					}
				}
			}
		}
		
		Err("No workspace currently selected!".to_owned())
	}
	
	
	// Save this workspace to the list of workspaces
	pub fn save(&self) -> Result<(), ErrorKind> {
		use std::fs::{File};
		use std::path::Path;
		
		use std::io::Write;
		if Path::new(WORKSPACES_FILE_PATH).exists() == false {
			let mut file = File::create(WORKSPACES_FILE_PATH).unwrap();
			file.write(DEFAULT_FILE.as_bytes()).unwrap();
		}
		
		
		let mut tree = XmlTree::from_file(WORKSPACES_FILE_PATH);
		
		for root in tree.roots_mut() {
			if root.tag() == "workspaces" {
				
				for elem in root.elements() {
					if elem.tag() == "workspace" {
						
						for attrib in elem.attributes() {
							if attrib.key == "name" && attrib.value == self.name {
								return Err(ErrorKind::AlreadyExists);
							}
						}
					}
				}
				
				root.add_element(self.as_xml_element());
			}
		}
		
		
		tree.write_to_file(WORKSPACES_FILE_PATH)
	}
	
	
	/// Return this workspace as an XmlElement 
	pub fn as_xml_element(&self) -> XmlElement {
		use xmltree::XmlAttribute;
		let attribs = vec![
			XmlAttribute{key: "name".to_owned(), value: self.name.clone()},
			XmlAttribute{key: "path".to_owned(), value: self.path.clone()},
		];
		
		XmlElement::new("workspace", "", attribs, Vec::new())
	}
	
	/// Create a workspace from an XmlElement
	pub fn from_xml_element(element: &XmlElement) -> Option<Workspace> {
		if element.tag() == "workspace" {
			let mut name = None;
			let mut path = None;
			
			for attrib in element.attributes() {
				if attrib.key == "name" {
					name = Some(attrib.value.clone());
				} else if attrib.key == "path" {
					path = Some(attrib.value.clone());
				} else {
					return None;
				}
			}
			
			if let Some(name) = name {
				if let Some(path) = path {
					return Some(Workspace{name, path});
				}
			}
		}		
		return None;
	} 
	
	/// Sets this workspace as the active one
	pub fn set_active(&self) {
		let mut tree = XmlTree::from_file(WORKSPACES_FILE_PATH);
		{
			let mut update_tree = || {
				for root in tree.roots_mut() {
					if root.tag() == "workspaces" {
						for elem in root.elements_mut() {
							if elem.tag() == "current" {
								for attrib in elem.attributes_mut() {
									if attrib.key == "name" {
										attrib.value = self.name.clone();
										return;
									}
								}
							}
						}
						use xmltree::XmlAttribute;
						root.add_element(XmlElement::new(
							"current",
							"",
							vec![XmlAttribute::new("name", &self.name)],
							Vec::new()
						));
						return;
					}
				}
			};
			update_tree();
		}
		
		tree.write_to_file(WORKSPACES_FILE_PATH).unwrap();
	}
	
	
	/// Adds a project to this workspace 
	pub fn add_project(&mut self, project: Project) -> Result<(), String> {
		let mut tree = XmlTree::from_file(&self.project_preferences_path());
				
		for root in tree.roots_mut() {
			if root.tag() == "projects" {
				root.merge_with_or_add(project.as_xml_element(), &mut |a, b| {
					let mut a_attribs = a.attributes();
					let mut b_attribs = b.attributes();
					
					loop {
						if let Some(a_attrib) = a_attribs.next() {
							if let Some(b_attrib) = b_attribs.next() {
								
								if a_attrib.key == "name" && b_attrib.key == "name" {
									if a_attrib.value == b_attrib.value {
										return true;
									}
								}
								
							} else {
								break;
							}
						} else {
							break;
						}
					}
					false
				});
				break;
			}
		}
		
		if let Err(_) = tree.write_to_file(&self.project_preferences_path()) {
			return Err("Failed to create new project".to_owned());
		}
		
		Ok(())
	}
	
	
	/// Remove a named workspace, 'purge' determines wheter to completely erase from disk
	pub fn remove(path: &str, purge: bool) -> Result<(), String> {
		use std::path::{MAIN_SEPARATOR};
		let path = if MAIN_SEPARATOR == '\\' {path.replace("/", "\\")} else {path.replace("\\", "/")};
		
		let name = (path.chars().rev().take_while(|c| { *c != '/' && *c != '\\' }).collect::<String>()).chars().rev().collect::<String>();
		
		let mut tree = XmlTree::from_file(WORKSPACES_FILE_PATH);
		
		let mut success = false;
		
		for root in tree.roots_mut() {
			if root.tag() == "workspaces" {
				
				let removed_item = root.remove(|elem|{
					if elem.tag() == "workspace" {
						let mut correct_name = false;
						let mut correct_path = false;
						for attrib in elem.attributes() {
							if attrib.key == "name" && attrib.value == name {
								correct_name = true;
							} else if attrib.key == "path" && attrib.value.contains(&path) {
								correct_path = true;
							}
						}
						if correct_name && correct_path {
							return true;
						}
					}
					false
				});
				
				if let Some(element) = removed_item {
					if purge {
						if let Some(workspace) = Workspace::from_xml_element(&element) {
							if let Err(e) = workspace.purge() {
								return Err(e);
							}
						}
					}
					success = true;
					break;
				}
			}
		}
		
		if success {
			if let Err(e) = tree.write_to_file(WORKSPACES_FILE_PATH) {
				match e {
					ErrorKind::AlreadyExists => return Err("Workspace with that name already exists".to_owned()),
					_ => return Err("Failed to remove workspace, failed to save changes".to_owned())
				}
			} else {
				return Ok(());
			}
		}
		
		Err(format!("No workspace with the name '{}'", name))
	}
	
	
	/// Purges the workspace from disk
	fn purge(&self) -> Result<(), String> {
		if let Err(_) = fs::remove_dir_all(&self.path) {
			return Err("Failed to purge workspace!".to_owned());
		}
		
		Ok(())
	}
	
	
	/// Creates the preference folder for a workspace
	fn setup_preferences(&self) -> Result<(), String> {
		use xmltree::XmlTree;
		
		// Create workspace folder
		if let Err(e) = fs::create_dir(self.workspace_preferences_folder_path()) {
			if e.kind() != ErrorKind::AlreadyExists {
				return Err("Failed to create workspace preference folder".to_owned());
			} 
		}
		
		// Create project preferences
		{
			let default_layout = "<projects></projects>";
			let tree = XmlTree::from_str(default_layout);
			if let Err(_) = tree.write_to_file(&self.project_preferences_path()) {
				return Err("Failed to create workspace project preferences".to_owned());
			}
		}
		
		Ok(())
	}
	
	/// Return the path to the project preferences
	fn workspace_preferences_folder_path(&self) -> String {
		use std::path::MAIN_SEPARATOR;
		self.path.clone() + &MAIN_SEPARATOR.to_string() + ".workspace"
	}
	
	/// Return the path to the project preferences
	fn project_preferences_path(&self) -> String {
		use std::path::MAIN_SEPARATOR;
		self.workspace_preferences_folder_path() + &MAIN_SEPARATOR.to_string() + "projects.xml"
	}
}