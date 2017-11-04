


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
}