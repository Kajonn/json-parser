# Json parser

Use string slices from original json to prevent any copies of strings.

Parser is optimistic and will return json from string with minor syntax errors. 

### Future development / TODOs

- Support file path or url as parse string input.
- Support store String in Json object in case where string lifetime cannot be garantueed (ex file read, http request)
- Improve number parsing
- Simplify access of json values