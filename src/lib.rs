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

//TODO enable [] access in Json

pub struct JsonBuilder<'a> {
    //currentState: JsonBuilderState,
    currentValue: Option<JsonValue<'a>>,
    currentTag: Option<&'a str>,
    
    inArray: bool,
    jsonBuildStack: Vec< JsonValue<'a> >,
}

impl<'a> JsonBuilder<'a> {

    pub fn parse(json_str: &'a str) ->Box<Json<'a>> {
        
        let mut builder = JsonBuilder {
            currentValue: None,
            currentTag: None,
            inArray: false,
            jsonBuildStack: Vec::new()
        };

        builder.jsonBuildStack.push( JsonValue::Object(Box::new(Json::new())));
        
        //Check that first character is { 
        //TODO start parse with the iterator
        builder.parse_next_value(json_str.chars()); 
        
        // Return parsed json
        // TODO check 
        if let Some(json) = builder.jsonBuildStack.pop() {

            if !builder.jsonBuildStack.is_empty() {
                //TODO WARN
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

            let currentObject = self.jsonBuildStack.first_mut().expect("No current json build");
            if let JsonValue::Array(array) = currentObject {
                
                array.push(self.currentValue.take().unwrap());
                //TODO could warn that no tag should be active

            } else if let JsonValue::Object(object)  = currentObject {
                if let Some(_tag) = &self.currentTag {
                    
                    println!("Setting {} ", _tag);
                    object.as_mut().fields.insert(self.currentTag.take().unwrap(), self.currentValue.take().unwrap());
            
                }
            }
        }
    }

    fn read_number(&mut self, st : Chars<'a>) -> (Chars<'a>, Option<f64>) {
        // Read to next space or ,  and return the value as double 
        if let Some(s) = st.as_str().split_once(|c| c==' ' || c==',') {
            return (s.1.chars(), Some(s.0.parse().expect("COULD NOT PARSE DOUBLE"))); //TODO MAY PANIC
        } else {
            return (st, None);
        }
    }
    

    fn read_string(&mut self, st : Chars<'a>) -> (Chars<'a>, Option<&'a str>) {
        // Read to next " and return string as string 
        if let Some(s) = st.as_str().split_once('"') {
            return (s.1.chars(), Some(s.0)); 
        } else {
            return (st, None);
        }
    }

    fn begin_next_object(&mut self) {
        if let Some(tag) = &self.currentTag  {        
            // Push new json to stack
            self.jsonBuildStack.push(JsonValue::Object(Box::new(Json::new())));
        } //else cannot set unnamed object TODO warning
    }

    fn begin_next_array(&mut self) {
        // Push new array to stack
        self.jsonBuildStack.push(JsonValue::Array(Vec::new()));        
    }

    fn end_current_object_or_array(&mut self){

        if self.jsonBuildStack.len() > 1 {

            if let Some(currentJson) = self.jsonBuildStack.pop() {    
            // Add to parent json or array            
                let currentObject = self.jsonBuildStack.first_mut().expect("No current json build");
                if let JsonValue::Array(array) = currentObject {
                    
                    array.push(currentJson);

                } else if let JsonValue::Object(object)  = currentObject {
                    
                    object.as_mut().fields.insert(self.currentTag.take().unwrap(), currentJson);
                
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
    } else {
        println!("Cannot parse string");
        return Box::new(Json{fields:HashMap::new()});
    }   
    // else if json_str.starts_with("http") {
    //     //If url read from request response
    // } else {
    //     // Try to parse file
    //     //If file path, 
    // }

}