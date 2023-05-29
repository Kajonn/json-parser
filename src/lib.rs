use std::{collections::HashMap, str::Chars};

// All strings are slices to prevent copies.
// Original json must be retained 
#[derive(Debug)]
pub enum JsonValue<'a> {
    Nothing,
    StringRef(&'a str),
    Integer(i32),
    Double(f64),
    Object(Box<Json<'a>>),
    Array(Vec<JsonValue<'a>>)
}

#[derive(Debug)]
pub struct Json<'a> {
    pub fields: HashMap<&'a str, JsonValue<'a>>
}

impl<'a> Json<'a> {
    fn new() -> Json<'a> {
        Json {
            fields: HashMap::new()
        }
    }
}

struct JsonBuilder<'a> {
    //currentState: JsonBuilderState,
    current_value: Option<JsonValue<'a>>,
    current_tag: Option<&'a str>,
    json_build_stack: Vec< JsonValue<'a> >,
    tag_stack: Vec<&'a str>,
}

impl<'a> JsonBuilder<'a> {

    pub fn parse(json_str: &'a str) ->Box<Json<'a>> {        
        return JsonBuilder::parse_internal(Box::new(Json::new()), json_str.chars());        
    }

    fn parse_internal(json: Box<Json<'a>>, json_str: Chars<'a>) ->Box<Json<'a>> {
        let mut builder = JsonBuilder {
            current_value: None,
            current_tag: None,
            json_build_stack: Vec::new(),
            tag_stack: Vec::new()
        };

        builder.json_build_stack.push( JsonValue::Object(json));
        
        builder.parse_next_value(json_str); 
        
        // Return parsed json
        if let Some(json) = builder.json_build_stack.pop() {

            if !builder.json_build_stack.is_empty() {
                println!("Could not parse!");
            }

            if let JsonValue::Object(jsonval) = json {
                return jsonval;
            }
        } 

        Box::new(Json::new())
    }

    //Find next value 
    fn parse_next_value(&mut self, mut st : Chars<'a>) -> Chars {
         //Original iterator is needed when parsing numbers
        let st_orig = st.clone();
        if let Some(c) = st.next() {
            dbg!(c);

            if c=='\n' || c==' ' ||  c==',' || c==':' {
                return self.parse_next_value(st); 
            } else if c=='"' {
                //Get string
                let (newst, tag_text_result) = self.read_string(st);
                if let Some(tag_text) = tag_text_result {
                    //Array values do not have tags 
                    if self.current_tag.is_none() && !self.is_in_array() {
                        // Found tag
                        self.current_tag = Some(tag_text);              
                    } else {
                        // Found string value
                        self.current_value = Some(JsonValue::StringRef(tag_text));
                        self.set_current_value(); //Sets value in object
                    }
                }
                // Call next value again with the new iterator
                return self.parse_next_value(newst);

            } else if c=='{' {

                // Create a new object
                self.begin_next_object();
                
                //Keep parsing
                return self.parse_next_value(st);

            } else if c =='}' {

                // Close active object
                self.end_current_object_or_array();

                //Keep parsing
                return self.parse_next_value(st);

            } else if c=='[' {

                self.begin_next_array();
                
                //Keep parsing
                return self.parse_next_value(st);

            } else if c==']' {
                
                self.end_current_object_or_array();
                
                //Keep parsing
                return self.parse_next_value(st);

            } else {
                // Problem with splitting the string and retaining end marker ] or } 
                // make parsing numbers more tricky. 
                // Evaluate number once, then iterator over it to reach next character. 
                if self.current_value.is_none() {
                    //Try to parse number if value hasn't been set
                    let number = self.read_number(st_orig);                    
                    if let Some(num) = number {
                        // Found number value
                        self.current_value = Some(JsonValue::Double(num));
                        self.set_current_value(); //Sets value in object                
                    }                
                }
                return self.parse_next_value(st);
            }
        } 
        // None, str end
        st
    }

    fn is_in_array(&self) -> bool {
        if let JsonValue::Array(_a) = self.json_build_stack.last().expect("No current json build") {
            true
        } else {
            false
        }
    }

    fn set_current_value(&mut self) {
        //println!("Set current value");
                    
        if let Some(_value) = &self.current_value  {

            dbg!(&self.current_value);

            let current_object = self.json_build_stack.last_mut().expect("No current json build");
            if let JsonValue::Array(array) = current_object {
                
                array.push(self.current_value.take().unwrap());

            } else if let JsonValue::Object(object)  = current_object {
                if let Some(_tag) = &self.current_tag {
                    //dbg!(&object);
                    //dbg!(_value);
                    //println!("tag {} ", _tag);
                    object.as_mut().fields.insert(self.current_tag.take().unwrap(), self.current_value.take().unwrap());
                    //dbg!(&object);                    
                } else {
                    println!("Missing tag for value in json object!");
                }
            }
        }
        self.current_tag=None;
        self.current_value=None;
    }

    fn read_number(&mut self, st : Chars<'a>) -> Option<f64> {
        // Read to next space or ,  and return the value as double 
        if let Some(s) = st.as_str().split_once(|c| c==' ' || c==',' || c=='}' || c==']') {
            return Some(s.0.parse().expect("COULD NOT PARSE DOUBLE")); //TODO MAY PANIC
        } else {
            println!("Could not parse number!");
            return None;
        }
    }
    

    fn read_string(&mut self, st : Chars<'a>) -> (Chars<'a>, Option<&'a str>) {
        // Read to next " and return string as string 
        if let Some(s) = st.as_str().split_once('"') {
            return (s.1.chars(), Some(s.0)); 
        } else {
            println!("Could not read string!");
            return (st, None);
        }
    }

    fn begin_next_object(&mut self) {
        if let Some(tag) = &self.current_tag  {        
            // Push new json to stack
            //println!("Add json {}", tag);
            self.json_build_stack.push(JsonValue::Object(Box::new(Json::new())));
            self.tag_stack.push(self.current_tag.take().unwrap());            
        } else if self.is_in_array() {
            self.json_build_stack.push(JsonValue::Object(Box::new(Json::new())));
        }else {
            println!("Could not add object with no tag!");
        }
    }

    fn begin_next_array(&mut self) {
        if let Some(tag) = &self.current_tag  {                
            // Push new array to stack
            self.json_build_stack.push(JsonValue::Array(Vec::new()));
            self.tag_stack.push(self.current_tag.take().unwrap());
        }else if self.is_in_array() {
            self.json_build_stack.push(JsonValue::Array(Vec::new()));            
        }
        else {
            println!("Could not add array with no tag!");
        }        
    }

    fn end_current_object_or_array(&mut self){
        if self.json_build_stack.len() > 1 {
            if let Some(current_json) = self.json_build_stack.pop() {
                // Add to parent json or array            
                let current_object = self.json_build_stack.last_mut().expect("No current json build");
                if let JsonValue::Array(array) = current_object {
                        
                    array.push(current_json);

                } else if let JsonValue::Object(object)  = current_object {
                    if let Some(tag) = self.tag_stack.pop() {    
                        //println!("Set json value {}", tag);
                        //dbg!(&object);
                        dbg!(&current_json);
    
                        object.as_mut().fields.insert(tag, current_json);
                        //dbg!(&object);
                        
                    } else {
                        println!("Missing tag when setting json!")
                    }
                }    
            }
        }
    }

}

pub fn parse(raw_str : &str) -> Box<Json> {
    
    let json_str = raw_str.trim();

    if json_str.starts_with('{') {
        return JsonBuilder::parse(json_str);
    }
     else {
        println!("Cannot read json!");
    }   
    
    Box::new(Json::new())
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_parse() {
        let json_str = r#"{"name":{"first":"jonas"}}"#;
        let parsed = parse(json_str);
        
        //assert_eq!(parsed.as_ref().fields.get("name").unwrap(), "jonas");
    }

    #[test]
    fn formatted_parse() {
        let json_str = r#"
        {
            "id": 1,
            "title": "iPhone 9",
            "description": "An apple mobile which is nothing like apple",
            "price": 549,
            "discountPercentage": 12.96,
            "rating": 4.69,
            "stock": 94,
            "brand": "Apple",
            "category": "smartphones",
            "thumbnail": "https://i.dummyjson.com/data/products/1/thumbnail.jpg",
            "images": [
              "https://i.dummyjson.com/data/products/1/1.jpg",
              "https://i.dummyjson.com/data/products/1/2.jpg",
              "https://i.dummyjson.com/data/products/1/3.jpg",
              "https://i.dummyjson.com/data/products/1/4.jpg",
              "https://i.dummyjson.com/data/products/1/thumbnail.jpg"
            ]
        }        
        "#;

        let parsed = parse(json_str);
        
        assert_eq!(parsed.fields.len(), 11);
        //assert_eq!(parsed.fields.get("images").unwrap()..len(), 5);
        // assert_eq!(parsed.fields.get("category").unwrap(), "smartphones");
    }

    #[test]
    fn complex_parse() {
        let json_str = r#"
        {"posts":[{"id":1,"title":"His mother had always taught him","body":"His mother had always taught him not to ever think of himself as better than others. He'd tried to live by this motto. He never looked down on those who were less fortunate or who had less money than him. But the stupidity of the group of people he was talking to made him change his mind.","userId":9,"tags":["history","american","crime"],"reactions":2}],"total":150,"skip":0,"limit":1}
        "#;

        let parsed = parse(json_str);
       
        assert_eq!(parsed.fields.len(), 4);
        //assert_eq!(parsed.fields.get("posts").unwrap().len(), 4);
    }
 }

 