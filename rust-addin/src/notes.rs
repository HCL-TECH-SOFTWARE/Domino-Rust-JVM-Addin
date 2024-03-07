//
// Copyright (c) 2023-2024 HCL America, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::{
    env::Args,
    ffi::{c_char, c_int, CStr, CString},
    path::{Path, PathBuf},
};

const MAXPATH: usize = 256;
const MAXENVVALUE: usize = 256;

#[link(name = "notes")]
extern "C" {
    // addin.h
    fn AddInLogMessageText(msg: *const c_char, additionalErrorCode: u16, args: ...);
    
    // global.h
    fn NotesInitExtended(argc: c_int, argv: *const *const c_char);
    fn NotesTerm();
    
    // mq.h
    fn MQOpen(QueueName: *const c_char, Options: u32, RetQueue: *mut u32) -> u16;
    
    // osenv.h
    fn OSGetEnvironmentString(
        variableName: *const c_char,
        retValueBuffer: *mut i8,
        bufferLength: u16,
    ) -> usize;
    
    // osfile.h
    fn OSGetExecutableDirectory(retPathName: *mut i8);
    fn OSGetDataDirectory(retPathName: *mut i8);
}

/// Initializes the Notes runtime using the provided argument array
pub fn init(argv: Args) {
    let args = argv
        .map(|arg| CString::new(arg).unwrap())
        .collect::<Vec<CString>>();
    let c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const c_char>>();
    unsafe {
        NotesInitExtended(c_args.len() as c_int, c_args.as_ptr());
    };
}

pub fn term() {
    unsafe {
        NotesTerm();
    }
}

pub fn exec_dir() -> PathBuf {
    let exec_dir: PathBuf;
    unsafe {
        let mut ret_path_name = vec![0; MAXPATH];
        let ptr = ret_path_name.as_mut_ptr() as *mut i8;
        OSGetExecutableDirectory(ptr);
        exec_dir = Path::new(CStr::from_ptr(ret_path_name.as_ptr()).to_str().unwrap()).to_owned();
    }
    return exec_dir;
}

pub fn data_dir() -> PathBuf {
    let data_dir: PathBuf;
    unsafe {
        let mut ret_path_name = vec![0; MAXPATH];
        let ptr = ret_path_name.as_mut_ptr() as *mut i8;
        OSGetDataDirectory(ptr);
        data_dir = Path::new(CStr::from_ptr(ret_path_name.as_ptr()).to_str().unwrap()).to_owned();
    }
    return data_dir;
}

/// Retrieves the named value from the Notes runtime's environment (notes.ini)
pub fn get_ini_var(var_name: &str) -> Option<String> {
    unsafe {
        let mut ret_value_buffer = vec![0; MAXENVVALUE];
        let ptr = ret_value_buffer.as_mut_ptr() as *mut i8;
        let found = OSGetEnvironmentString(
            CString::new(var_name).unwrap().into_raw(),
            ptr,
            MAXENVVALUE.try_into().unwrap(),
        );
        if found != 0 {
            return Option::Some(
                CStr::from_ptr(ret_value_buffer.as_ptr())
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        } else {
            return Option::None;
        }
    }
}

pub fn addin_log(msg: String) {
    unsafe {
        AddInLogMessageText(CString::new(msg).unwrap().into_raw(), 0);
    }
}

pub fn queue_exists(queue_name: &str) -> bool {
  let mut queue: u32 = 0;
  unsafe {
    let status = MQOpen(CString::new(queue_name).unwrap().into_raw(), 0, &mut queue);
    return status == 0;
  }
}
