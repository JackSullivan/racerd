#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;
extern crate hyper;

use std::io::Write;
use std::collections::BTreeMap;

use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, Error};
use serde_json::{Value, from_str};

use hyper::Server;
use hyper::server::{Request, Response};
use hyper::net::Fresh;

// {
//     "column_num": 1,
//     "event_name": "FileReadyToParse",
//     "file_data": {
//         "<absolute path of file>": {
//             "contents": "<file contents with escaped newlines>",
//             "filetypes": [
//                 "<the filetype; in our case, rust.>"
//             ]
//         }
//     },
//     "filepath": "<absolute path of file>",
//     "line_num": 1
// }

#[derive(Debug, Serialize)]
struct FileData {
    absolute_path: String,
    contents: String,
    filetypes: Vec<String>,
}

impl Deserialize for FileData {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        match BTreeMap::<String, BTreeMap<String, Value>>::deserialize(deserializer) {
            Ok(fd_map) => if fd_map.keys().count() == 1 {
                match fd_map.keys().next() {
                    Some(k) => {
                        let fd_inner = fd_map.get(k).unwrap();
                        match (fd_inner.get("contents"), fd_inner.get("filetypes")) {
                            (Some(&Value::String(ref cont)), Some(&Value::Array(ref ftypes))) => {
                                let ftype_strings:Vec<_> = ftypes.into_iter().flat_map(|ft| {
                                    match ft {
                                        &Value::String(ref ft_s) => Some(ft_s.clone()),
                                        _ => None
                                    }
                                }).collect();
                                Ok(FileData{ absolute_path: k.clone(), contents: cont.clone(), filetypes: ftype_strings, })
                            },
                            _ => Err(D::Error::missing_field("we got problems"))
                        }
                    },
                    None => Err(D::Error::missing_field("we got problems"))
                }   
            } else {
                Err(D::Error::missing_field("we got problems"))
            },
            Err(err) => Err(err)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct EventNotification {
    column_num: i32,
    event_name: String,
    file_data: FileData,
    filepath: String,
    line_num: i32,
}

fn main() {
    println!("Starting Server");
    let event_notification_json = r#"{
        "column_num": 1,
        "event_name": "FileReadyToParse",
        "file_data": {
            "/Users/jwilm/code/ycmd/examples/samples/some_javascript.js": {
                "contents": "// Copyright (C) 2014  Google Inc.\n//\n// Licensed under the Apache License, Version 2.0 (the \"License\");\n// you may not use this file except in compliance with the License.\n// You may obtain a copy of the License at\n//\n//     http://www.apache.org/licenses/LICENSE-2.0\n//\n// Unless required by applicable law or agreed to in writing, software\n// distributed under the License is distributed on an \"AS IS\" BASIS,\n// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n// See the License for the specific language governing permissions and\n// limitations under the License.\n\n(function() {\n  var x = 10;\n  var y = 15;\n  var foobar = x + y;\n  var foozoo = x + y;\n  // location after second 'o' is line 24, column 6\n  foo\n});\n\n",
                "filetypes": [
                    "javascript"
                ]
            }
        },
        "filepath": "/Users/jwilm/code/ycmd/examples/samples/some_javascript.js",
        "line_num": 1
    }"#;

    
    let en:EventNotification = from_str(&event_notification_json).unwrap();
    println!("{:?}", en);
}
