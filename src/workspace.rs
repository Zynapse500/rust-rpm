
use std::fs;
use std::io::ErrorKind;

use xmltree::{XmlTree, XmlElement};

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
		let name = (&path.chars().rev().take_while(|c| { *c != '/' && *c != '\\' }).collect::<String>()).chars().rev().collect();
		
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
	
	
	/// Remove a named workspace, 'purge' determines wheter to completely erase from disk
	pub fn remove(name: &str, purge: bool) -> Result<(), String> {
		
		let mut tree = XmlTree::from_file(WORKSPACES_FILE_PATH);
		
		let mut success = false;
		
		for root in tree.roots_mut() {
			if root.tag() == "workspaces" {
				if let Some(element) = root.remove(|elem|{
					if elem.tag() == "workspace" {
						for attrib in elem.attributes() {
							if attrib.key == "name" && attrib.value == name {
								return true;
							}
						}
					}
					false
				}) {
					success = true;
					
					if purge {
						// fs::remove_dir();
					}
					
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
}