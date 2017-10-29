
use xmltree::{XmlElement, XmlAttribute};

pub struct Project {
	name: String,
	projects: Vec<Project>
}


impl Project {
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
	
	
	/// Creates a XmlElement form this project
	pub fn as_xml_element(&self) -> XmlElement {
		let attributes = vec![
			XmlAttribute::new("name", &self.name)
		];
		
		let mut sub_elements = Vec::new();
		
		for project in self.projects.iter() {
			sub_elements.push(project.as_xml_element());
		}
		
		XmlElement::new("project", "", attributes, sub_elements)
	}
}