
use quick_xml::writer::Writer;
use quick_xml::reader::Reader;
use quick_xml::events::{Event, BytesStart, BytesEnd};

use std::io::{Read, Write, ErrorKind};
use std::fs::{OpenOptions, File};
use std::str::from_utf8;

use std::slice::{Iter, IterMut};

pub struct XmlTree{
	root: XmlElement
}

impl XmlTree {
	
	/// Creates a XmlTree from text in string
	pub fn from_str(text: &str) -> XmlTree {
		let mut root = XmlElement{
			tag: "".to_owned(),
			text: "".to_owned(),
			attributes: Vec::new(),
			sub_elements: Vec::new()
		};
		
		let mut reader = Reader::from_str(text);
		reader.trim_text(true);
		
		root.build_from_reader(&mut reader);
		
		XmlTree{root}
	}
	
	
	/// Creates a XmlTree from a file at path
	pub fn from_file(path: &str) -> XmlTree {
		let mut xml: String = "".to_owned();
		
		{
			let mut file: File = OpenOptions::new()
			.read(true)
			.open(path).unwrap();
			
			file.read_to_string(&mut xml).unwrap();
		}
		
		XmlTree::from_str(&xml)
	}
	
	
	/// Returns the XmlTree as a string
	pub fn as_string(&self) -> String {
		use std::io::Cursor;
		let mut writer = Writer::new(Cursor::new(Vec::new()));
		
		for elem in self.root.sub_elements.iter() {
			elem.write_to_writer(&mut writer);
		}
		
		let result_buffer = writer.into_inner().into_inner();
		let result_string = from_utf8(&result_buffer).unwrap().to_owned();
		
		result_string
	}
	
	
	/// Writes the XmlTree to file
	pub fn write_to_file(&self, path: &str) -> Result<(), ErrorKind> {
		let string = self.as_string();

		let string = indent(&string);

		let file = File::create(path);
		
		return match file {
			Err(e) => Err(e.kind()),
			Ok(mut file) => {
				match file.write(string.as_bytes()) {
					Err(e) => Err(e.kind()),
					_ => Ok(())
				}
			}
		}
	}
	
	
	/// Return the roots of the xml
	pub fn roots(&self) -> Iter<XmlElement> {
		self.root.sub_elements.iter()
	}
	
	/// Return the mutable roots of the xml
	pub fn roots_mut(&mut self) -> IterMut<XmlElement> {
		self.root.sub_elements.iter_mut()
	}
}


pub struct XmlElement {
	tag: String,
	text: String,
	
	attributes: Vec<XmlAttribute>,
	sub_elements: Vec<XmlElement>
}


impl XmlElement {
	
	/// Create a new element
	pub fn new(tag: &str, text: &str, attributes: Vec<XmlAttribute>, sub_elements: Vec<XmlElement>) -> XmlElement {
		XmlElement {
			tag: tag.to_owned(),
			text: text.to_owned(),
			attributes,
			sub_elements
		}
	} 
	
	pub fn from_start(start: BytesStart) -> XmlElement {
		let name = from_utf8(start.name()).unwrap().to_owned();
		
		let attributes = {
			let mut attribs = Vec::new();
			for attr in start.attributes() {
				let attrib = attr.unwrap();
				
				let key = from_utf8(attrib.key).unwrap().to_owned();
				let value = from_utf8(attrib.value).unwrap().to_owned();
				
				attribs.push(XmlAttribute{key, value});
			}
			attribs
		};
		
		
		XmlElement {
			tag: name,
			text: "".to_owned(),
			attributes,
			sub_elements: Vec::new()
		}
	}
	
	
	/// Construct this element from XmlReader
	pub fn build_from_reader(&mut self, reader: &mut Reader<&[u8]>) {
		let mut buf = Vec::new();
		loop {
			match reader.read_event(&mut buf) {
				Ok(Event::Start(e)) => {
					let mut next_element = XmlElement::from_start(e);
					next_element.build_from_reader(reader);
					self.sub_elements.push(next_element);
				}
				Ok(Event::End(_)) => {
					break;
				}
				Ok(Event::Text(text)) => {
					use std::ops::Deref;
					self.text = from_utf8(text.deref()).unwrap().to_owned();
				}
				Ok(Event::Eof) => break,
				Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
				_ => ()
			}
			buf.clear();
		}
	}
	
	
	/// Writes this element and all it's subelements to a writer
	pub fn write_to_writer<W: Write>(&self, writer: &mut Writer<W>)
	{
		let mut start = BytesStart::borrowed(self.tag.as_bytes(), self.tag.len());
		let end = BytesEnd::borrowed(self.tag.as_bytes());
		
		for attrib in self.attributes.iter() {
			start.push_attribute((&attrib.key[..], &attrib.value[..]));
		}
		
		writer.write_event(Event::Start(start)).unwrap();
		
		for elem in self.sub_elements.iter() {
			elem.write_to_writer(writer);
		}
		
		writer.write_event(Event::End(end)).unwrap();
	}
	
	
	
	/// Removes the first sub_element that fulfills the predicate
	/// Returns the removed element, if there is one
	pub fn remove<F: FnMut(&XmlElement) -> bool>(&mut self, mut predicate: F) -> Option<XmlElement> {
		
		let mut remove_index = None;
		for (index, elem) in self.sub_elements.iter().enumerate() {
			if predicate(elem) {
				remove_index = Some(index);
				break;
			}
		}
		
		if let Some(index) = remove_index {
			return Some(self.sub_elements.remove(index));
		}
		
		None
	}
	
	
	
	/// Iterator over sub-elements
	pub fn elements(&self) -> Iter<XmlElement> {
		self.sub_elements.iter()
	}
	
	/// Iterator over sub-elements
	pub fn elements_mut(&mut self) -> IterMut<XmlElement> {
		self.sub_elements.iter_mut()
	}
	
	/// Iterator over attributes
	pub fn attributes(&self) -> Iter<XmlAttribute> {
		self.attributes.iter()
	}
	
	/// Iterator over sub-elements
	pub fn attributes_mut(&mut self) -> IterMut<XmlAttribute> {
		self.attributes.iter_mut()
	}
	
	/// Return the tag of the element
	pub fn tag(&self) -> &str {
		&self.tag
	}
	
	
	/// Add element to sub-elements
	pub fn add_element(&mut self, element: XmlElement) {
		self.sub_elements.push(element);
	}
	
	
	/// Adds an element to the sub-elements, if the predicate is true, merges them recursively instead
	pub fn merge_with_or_add<F: FnMut(&XmlElement, &XmlElement) -> bool>(&mut self, element: XmlElement, predicate: &mut F) {
		for elem in self.sub_elements.iter_mut() {
			if predicate(elem, &element) {
				// merge
				for sub_elem in element.sub_elements {
					elem.merge_with_or_add(sub_elem, predicate);
				}
				
				return;
			}
		}
		
		self.add_element(element);
	}
}



pub struct XmlAttribute {
	pub key: String,
	pub value: String,
}


impl XmlAttribute {
	pub fn new(key: &str, value: &str) -> XmlAttribute {
		XmlAttribute {
			key: key.to_owned(),
			value: value.to_owned(),
		}
	}
}



/// Indents a string, xml-style
fn indent(text: &str) -> String {
	let mut result = "".to_owned();
	
	let mut indentation = 0;
	let indent = |count| {
		let mut result = "".to_owned();
		for _ in 0..count {
			 result += "  "
		}
		result
	};
	
	let mut ch = text.chars();
	loop {
		match ch.next() {
			Some('<') => {
				if let Some(ch) = ch.next() {
					if ch == '/' {
						indentation -= 1;
						result += &indent(indentation);
					} else {
						result += &indent(indentation);
						indentation += 1;
					}
					result += "<";
					result += &ch.to_string();
				} else {break}
			}
			Some('>') => result += ">\n",
			Some(a) => result += &a.to_string(),
			None => break
		}
	}
	
	result
}
