// Copyright 2022 Datafuse Labs.
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

use std::io::Result;

use anyhow::anyhow;

use crate::error::other;
use crate::error::ObjectError;
use crate::multipart::ObjectPart;
use crate::ops::Operation;

/// Args for `create_multipart` operation.
///
/// The path must be normalized.
#[derive(Debug, Clone, Default)]
pub struct OpCreateMultipart {
    path: String,
}

impl OpCreateMultipart {
    /// Create a new `OpCreateMultipart`.
    ///
    /// If input path is not a file path, an error will be returned.
    pub fn new(path: &str) -> Result<Self> {
        if path.ends_with('/') {
            return Err(other(ObjectError::new(
                Operation::CreateMultipart,
                path,
                anyhow!("Is a directory"),
            )));
        }

        Ok(Self {
            path: path.to_string(),
        })
    }

    /// Get path from option.
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Args for `write_multipart` operation.
///
/// The path must be normalized.
#[derive(Debug, Clone, Default)]
pub struct OpWriteMultipart {
    path: String,
    upload_id: String,
    part_number: usize,
    size: u64,
}

impl OpWriteMultipart {
    /// Create a new `OpWriteMultipart`.
    ///
    /// If input path is not a file path, an error will be returned.
    pub fn new(path: &str, upload_id: &str, part_number: usize, size: u64) -> Result<Self> {
        if path.ends_with('/') {
            return Err(other(ObjectError::new(
                Operation::WriteMultipart,
                path,
                anyhow!("Is a directory"),
            )));
        }

        Ok(Self {
            path: path.to_string(),
            upload_id: upload_id.to_string(),
            part_number,
            size,
        })
    }

    /// Get path from option.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get upload_id from option.
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }

    /// Get part_number from option.
    pub fn part_number(&self) -> usize {
        self.part_number
    }

    /// Get size from option.
    pub fn size(&self) -> u64 {
        self.size
    }
}

/// Args for `complete_multipart` operation.
///
/// The path must be normalized.
#[derive(Debug, Clone, Default)]
pub struct OpCompleteMultipart {
    path: String,
    upload_id: String,
    parts: Vec<ObjectPart>,
}

impl OpCompleteMultipart {
    /// Create a new `OpCompleteMultipart`.
    ///
    /// If input path is not a file path, an error will be returned.
    pub fn new(path: &str, upload_id: &str, parts: Vec<ObjectPart>) -> Result<Self> {
        if path.ends_with('/') {
            return Err(other(ObjectError::new(
                Operation::CompleteMultipart,
                path,
                anyhow!("Is a directory"),
            )));
        }

        Ok(Self {
            path: path.to_string(),
            upload_id: upload_id.to_string(),
            parts,
        })
    }

    /// Get path from option.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get upload_id from option.
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }

    /// Get parts from option.
    pub fn parts(&self) -> &[ObjectPart] {
        &self.parts
    }
}

/// Args for `abort_multipart` operation.
///
/// The path must be normalized.
#[derive(Debug, Clone, Default)]
pub struct OpAbortMultipart {
    path: String,
    upload_id: String,
}

impl OpAbortMultipart {
    /// Create a new `OpAbortMultipart`.
    ///
    /// If input path is not a file path, an error will be returned.
    pub fn new(path: &str, upload_id: &str) -> Result<Self> {
        if path.ends_with('/') {
            return Err(other(ObjectError::new(
                Operation::AbortMultipart,
                path,
                anyhow!("Is a directory"),
            )));
        }

        Ok(Self {
            path: path.to_string(),
            upload_id: upload_id.to_string(),
        })
    }

    /// Get path from option.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get upload_id from option.
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
}
