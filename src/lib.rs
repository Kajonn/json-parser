use std::{collections::HashMap, str::Chars};
use std::fs;

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
    //Optionally holds the raw string which is used for slices
    pub raw_str: Option<String>,
    pub fields: HashMap<&'a str, JsonValue<'a>>
}

impl<'a> Json<'a> {
    fn new() -> Json<'a> {
        Json {
            raw_str: None,
            fields: HashMap::new()
        }
    }
    fn new(raw_json_str: String) -> Json<'a> {
        Json {
            raw_str: Some(raw_json_str),
            fields: HashMap::new()
        }
    }
}

pub struct JsonBuilder<'a> {
    //currentState: JsonBuilderState,
    currentValue: Option<JsonValue<'a>>,
    currentTag: Option<&'a str>,
    
    inArray: bool,
    jsonBuildStack: Vec< JsonValue<'a> >,
    tagStack: Vec<&'a str>,
}

impl<'a> JsonBuilder<'a> {
    // TODO add parse with consumed string
    pub fn parse(json_str: String) ->Box<Json<'a>> {
        // Move string into root json
        let json_root = Box::new(Json::new(json_str));     
        // Use this string 
        let json_str = builder.jsonBuildStack.first().raw_str().chars();

        return parse(json_root, json_str)
    }
    pub fn parse(json_str: &'a str) ->Box<Json<'a>> {        
        return parse(JsonValue::Object(Box::new(Json::new())), json_str);        
    }

    pub fn parse(json: Box<Json<'a>>, json_str: Chars<'a>) ->Box<Json<'a>> {
        let mut builder = JsonBuilder {
            currentValue: None,
            currentTag: None,
            inArray: false,
            jsonBuildStack: Vec::new(),
            tagStack: Vec::new()
        };

        builder.jsonBuildStack.push( JsonValue::Object(json));
        builder.parse_next_value(json_str); 
        
        // Return parsed json
        if let Some(json) = builder.jsonBuildStack.pop() {

            if !builder.jsonBuildStack.is_empty() {
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
        if let Some(c) = st.next() {
            if c=='\n' || c==' ' ||  c==',' || c==':' {
                return self.parse_next_value(st); 
            } else if c=='"' {
                //Get string
                let (newst, text) = self.read_string(st);
                if let Some(t) = text {
                    //Array values do not have tags 
                    if self.currentTag.is_none() && !self.inArray {
                        // Found tag
                        self.currentTag = text;              
                    } else {
                        // Found string value
                        self.currentValue = Some(JsonValue::StringRef(text.unwrap()));
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

                //Get number
                let (newst, number) = self.read_number(st);
                if let Some(num) = number {
                    // Found number value
                    self.currentValue = Some(JsonValue::Double(num));
                    self.set_current_value(); //Sets value in object                
                }
                // Call next value again with the new iterator
                return self.parse_next_value(newst);
            }
        } 
        // None, str end
        
        st
    }

    fn set_current_value(&mut self) {
        println!("Set current value");
                    
        if let Some(_value) = &self.currentValue  {

            let currentObject = self.jsonBuildStack.last_mut().expect("No current json build");
            if let JsonValue::Array(array) = currentObject {
                
                array.push(self.currentValue.take().unwrap());

            } else if let JsonValue::Object(object)  = currentObject {
                if let Some(_tag) = &self.currentTag {
                    dbg!(&object);
                    dbg!(_value);
                    println!("Setting {} ", _tag);
                    object.as_mut().fields.insert(self.currentTag.take().unwrap(), self.currentValue.take().unwrap());
                    dbg!(&object);
                    
                } else {
                    println!("Missing tag for value in json object!");
                }
            }
        }
    }

    fn read_number(&mut self, st : Chars<'a>) -> (Chars<'a>, Option<f64>) {
        // Read to next space or ,  and return the value as double 
        if let Some(s) = st.as_str().split_once(|c| c==' ' || c==',') {
            return (s.1.chars(), Some(s.0.parse().expect("COULD NOT PARSE DOUBLE"))); //TODO MAY PANIC
        } else {
            println!("Could not parse number!");
            return (st, None);
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
        if let Some(tag) = &self.currentTag  {        
            // Push new json to stack
            println!("Add json {}", tag);
            self.jsonBuildStack.push(JsonValue::Object(Box::new(Json::new())));
            self.tagStack.push(self.currentTag.take().unwrap());            
        } else {
            println!("Could not add object with no tag!");
        }
    }

    fn begin_next_array(&mut self) {
        if let Some(tag) = &self.currentTag  {                
            // Push new array to stack
            self.jsonBuildStack.push(JsonValue::Array(Vec::new()));
            self.tagStack.push(self.currentTag.take().unwrap());
        } else {
            println!("Could not add array with no tag!");
        }        
    }

    fn end_current_object_or_array(&mut self){

        if self.jsonBuildStack.len() > 1 {
            if let Some(currentJson) = self.jsonBuildStack.pop() {
                // Add to parent json or array            
                    let currentObject = self.jsonBuildStack.last_mut().expect("No current json build");
                    if let JsonValue::Array(array) = currentObject {
                        
                        array.push(currentJson);

                    } else if let JsonValue::Object(object)  = currentObject {
                        if let Some(tag) = self.tagStack.pop() {    
                            println!("Set json value {}", tag);
                            dbg!(&object);
                            dbg!(&currentJson);
        
                            object.as_mut().fields.insert(tag, currentJson);
                            dbg!(&object);
                            
                    }
                }    
            }
        }
    }

}

// Takes any string and handles it as expected
pub fn parse(raw_str : &str) -> Box<Json> {
    
    let json_str = raw_str.trim();

    if json_str.starts_with('{') {
        //If begin with { parse directly
        return JsonBuilder::parse(json_str);
    } else if json_str.starts_with("http") {
        //TODO make http request
    } else {
        // Else parse file
        let json_string : String = fs::read_to_string(raw_str).expect("Cannot read file");
        return JsonBuilder::parse(json_string);
    }   
    
}


#[cfg(test)]
mod tests {
    #[test]
    fn simple_parse() {
        let json_str = r#"{"name":{"first":"jonas"}}"#;
        let parsed = json_parser::parse(json_str);
        
        assert_eq!(parsed.fields.get("name").unwrap(), "jonas");
    }


    fn complex_parse() {
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

        let parsed = json_parser::parse(json_str);
       
        assert_eq!(parsed.fields.len(), 11);
        assert_eq!(parsed.fields.get("images").unwrap().len(), 5);
        assert_eq!(parsed.fields.get("category").unwrap(), "smartphones");
    }

    fn post_parse() {
        let json_parse = r#"
        {"posts":[{"id":1,"title":"His mother had always taught him","body":"His mother had always taught him not to ever think of himself as better than others. He'd tried to live by this motto. He never looked down on those who were less fortunate or who had less money than him. But the stupidity of the group of people he was talking to made him change his mind.","userId":9,"tags":["history","american","crime"],"reactions":2}],"total":150,"skip":0,"limit":1}
        "#;

        let parsed = json_parser::parse(json_str);
       
        assert_eq!(parsed.fields.len(), 4);
        assert_eq!(parsed.fields.get("posts").unwrap().len(), 4);
    }
 }

 