// Copyright (c) 2020 lumi <lumi@pew.im>
// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2020 Bastien Orivel <eijebong+minidom@bananium.fr>
// Copyright (c) 2020 Astro <astro@spaceboyz.net>
// Copyright (c) 2020 Maxime “pep” Buquet <pep@bouah.net>
// Copyright (c) 2020 Matt Bilker <me@mbilker.us>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Provides an error type for this crate.

use std::convert::From;
use std::error::Error as StdError;

/// Our main error type.
#[derive(Debug)]
pub enum Error {
    /// Error from rxml parsing or writing
    XmlError(rxml::Error),

    /// An error which is returned when the end of the document was reached prematurely.
    EndOfDocument,

    /// An error which is returned when an element being serialized doesn't contain a prefix
    /// (be it None or Some(_)).
    InvalidPrefix,

    /// An error which is returned when an element doesn't contain a namespace
    MissingNamespace,

    /// An error which is returned when a prefixed is defined twice
    DuplicatePrefix,
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Error::XmlError(e) => Some(e),
            Error::EndOfDocument => None,
            Error::InvalidPrefix => None,
            Error::MissingNamespace => None,
            Error::DuplicatePrefix => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::XmlError(e) => write!(fmt, "XML error: {}", e),
            Error::EndOfDocument => {
                write!(fmt, "the end of the document has been reached prematurely")
            }
            Error::InvalidPrefix => write!(fmt, "the prefix is invalid"),
            Error::MissingNamespace => write!(fmt, "the XML element is missing a namespace",),
            Error::DuplicatePrefix => write!(fmt, "the prefix is already defined"),
        }
    }
}

impl From<rxml::Error> for Error {
    fn from(err: rxml::Error) -> Error {
        Error::XmlError(err)
    }
}

impl From<rxml::error::XmlError> for Error {
    fn from(err: rxml::error::XmlError) -> Error {
        Error::XmlError(err.into())
    }
}

/// Our simplified Result type.
pub type Result<T> = ::std::result::Result<T, Error>;
