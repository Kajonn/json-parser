use std::{collections::HashMap, str::Chars};

// All strings are slices to prevent copies.
// Original json must be retained 
#[derive(Debug)]
pub enum JsonValue<'a> {
    Nothing,
    StringRef(&'a str),
    Integer(i64),
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

        //Advance iterator past first {
        let mut tempst = json_str.clone();
        while let Some(c) = tempst.next() {
            if c=='{' {
                break;
            }
        }
        
        let _end_str = builder.parse_next_value(tempst); 
        
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
        if let Some(c) = st.next() {
            //dbg!(c);

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
                    } else if self.current_value.is_none() {
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
                if self.current_value.is_none() {
                    //Try to parse number if value hasn't been set
                    let newst = self.read_number_value(c,st);                    
                    return self.parse_next_value(newst);
                } else {
                    println!("Unexpected char {}!", c); 
                    return self.parse_next_value(st);
                }
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
           
            let current_object = self.json_build_stack.last_mut().expect("No current json build");
            if let JsonValue::Array(array) = current_object {
                
                array.push(self.current_value.take().unwrap());

            } else if let JsonValue::Object(object)  = current_object {
                if let Some(_tag) = &self.current_tag {
                    object.as_mut().fields.insert(self.current_tag.take().unwrap(), self.current_value.take().unwrap());
                } else {
                    println!("Missing tag when setting value in json!")
                }
            }
        }
        self.current_tag=None;
        self.current_value=None;
    }

    fn read_number_value(&mut self, first_c  : char, st : Chars<'a>) -> Chars<'a> {
        //Parse st as a number, return number as json value
        //Use first_c as first char in number as st has already been advanced
        let mut tempst = st.clone();
        let mut number_str = first_c.to_string();
        while let Some(c) = tempst.next() {
            if c.is_numeric() || c=='.' || c=='-' {
                number_str.push(c);
            } else {
                break;
            }
        }
        if let Ok(num) = number_str.parse::<i64>() {
            self.current_value = Some(JsonValue::Integer(num));
            self.set_current_value();
        } else if let Ok(num) = number_str.parse::<f64>() {
            self.current_value = Some(JsonValue::Double(num));
            self.set_current_value();
        } else {
            println!("Could not parse number!");
        }
        //Slice st to remove number
        st.as_str().split_at(number_str.len()-1).1.chars()   
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
        if let Some(_tag) = &self.current_tag  {        
            // Push new json to stack
            //println!("Add json {}", tag);
            self.json_build_stack.push(JsonValue::Object(Box::new(Json::new())));
            self.tag_stack.push(self.current_tag.take().unwrap());            
        } else if self.is_in_array() {
            self.json_build_stack.push(JsonValue::Object(Box::new(Json::new())));
        } else {
            println!("Could not add object with no tag!");
        }
    }

    fn begin_next_array(&mut self) {
        if let Some(_tag) = &self.current_tag  {                
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
       
                        object.as_mut().fields.insert(tag, current_json);
                        
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
        let json_str = r#"{"name":"jonas","number":10}"#;
        let parsed = parse(json_str);
        
        dbg!(&parsed);

        assert!(matches!( parsed.fields.get("name").unwrap(), JsonValue::StringRef("jonas") ));
        
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
        dbg!(&parsed);
        assert_eq!(parsed.fields.len(), 11);
        
        assert!(matches!( parsed.fields.get("category").unwrap(), JsonValue::StringRef("smartphones") ));
        assert!(matches!( parsed.fields.get("rating").unwrap(), JsonValue::Double(4.69) ));
        
    }

    #[test]
    fn complex_parse() {
        let json_str = r#"
        {"posts":[{"id":1,"title":"His mother had always taught him","body":"His mother had always taught him not to ever think of himself as better than others. He'd tried to live by this motto. He never looked down on those who were less fortunate or who had less money than him. But the stupidity of the group of people he was talking to made him change his mind.","userId":9,"tags":["history","american","crime"],"reactions":2}],"total":150,"skip":0,"limit":1}
        "#;

        let parsed = parse(json_str);
        dbg!(&parsed);
        assert_eq!(parsed.fields.len(), 4);
        
        assert!(matches!( parsed.fields.get("total").unwrap(), JsonValue::Integer(150) ));
    
    }
 }

 