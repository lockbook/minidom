// Copyright (c) 2022 Astro <astro@spaceboyz.net>

//! SAX events to DOM tree conversion

use crate::prefixes::{Prefix, Prefixes};
use crate::{Element, Error};
use rxml::RawEvent;
use std::collections::BTreeMap;

/// Tree-building parser state
pub struct TreeBuilder {
    next_tag: Option<(Prefix, String, Prefixes, BTreeMap<String, String>)>,
    /// Parsing stack
    stack: Vec<Element>,
    /// Namespace set stack by prefix
    prefixes_stack: Vec<Prefixes>,
    /// Document root element if finished
    pub root: Option<Element>,
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeBuilder {
    /// Create a new one
    pub fn new() -> Self {
        TreeBuilder {
            next_tag: None,
            stack: vec![],
            prefixes_stack: vec![],
            root: None,
        }
    }

    /// Allow setting prefixes stack.
    ///
    /// Useful to provide knowledge of namespaces that would have been declared on parent elements
    /// not present in the reader.
    pub fn with_prefixes_stack(mut self, prefixes_stack: Vec<Prefixes>) -> Self {
        self.prefixes_stack = prefixes_stack;
        self
    }

    /// Stack depth
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Get the top-most element from the stack but don't remove it
    pub fn top(&mut self) -> Option<&Element> {
        self.stack.last()
    }

    /// Pop the top-most element from the stack
    fn pop(&mut self) -> Option<Element> {
        self.prefixes_stack.pop();
        self.stack.pop()
    }

    /// Unshift the first child of the top element
    pub fn unshift_child(&mut self) -> Option<Element> {
        let depth = self.stack.len();
        if depth > 0 {
            self.stack[depth - 1].unshift_child()
        } else {
            None
        }
    }

    /// Lookup XML namespace declaration for given prefix (or no prefix)
    fn lookup_prefix(&self, prefix: &Option<String>) -> Option<&str> {
        for nss in self.prefixes_stack.iter().rev() {
            if let Some(ns) = nss.get(prefix) {
                return Some(ns);
            }
        }

        None
    }

    fn process_end_tag(&mut self) -> Result<(), Error> {
        if let Some(el) = self.pop() {
            if self.depth() > 0 {
                let top = self.stack.len() - 1;
                self.stack[top].append_child(el);
            } else {
                self.root = Some(el);
            }
        }

        Ok(())
    }

    fn process_text(&mut self, text: String) {
        if self.depth() > 0 {
            let top = self.stack.len() - 1;
            self.stack[top].append_text_node(text);
        }
    }

    /// Process a Event that you got out of a RawParser
    pub fn process_event(&mut self, event: RawEvent) -> Result<(), Error> {
        match event {
            RawEvent::XmlDeclaration(_, _) => {}

            RawEvent::ElementHeadOpen(_, (prefix, name)) => {
                self.next_tag = Some((
                    prefix.map(|prefix| prefix.as_str().to_owned()),
                    name.as_str().to_owned(),
                    Prefixes::default(),
                    BTreeMap::new(),
                ))
            }

            RawEvent::Attribute(_, (prefix, name), value) => {
                if let Some((_, _, ref mut prefixes, ref mut attrs)) = self.next_tag.as_mut() {
                    match (prefix, name) {
                        (None, xmlns) if xmlns == "xmlns" => prefixes.insert(None, value),
                        (Some(xmlns), prefix) if xmlns.as_str() == "xmlns" => {
                            prefixes.insert(Some(prefix.as_str().to_owned()), value);
                        }
                        (Some(prefix), name) => {
                            attrs.insert(format!("{}:{}", prefix, name), value.as_str().to_owned());
                        }
                        (None, name) => {
                            attrs.insert(name.as_str().to_owned(), value.as_str().to_owned());
                        }
                    }
                }
            }

            RawEvent::ElementHeadClose(_) => {
                if let Some((prefix, name, prefixes, attrs)) = self.next_tag.take() {
                    self.prefixes_stack.push(prefixes.clone());

                    let namespace = self
                        .lookup_prefix(&prefix.clone().map(|prefix| prefix.as_str().to_owned()))
                        .ok_or(Error::MissingNamespace)?
                        .to_owned();
                    let el =
                        Element::new(name.as_str().to_owned(), namespace, prefixes, attrs, vec![]);
                    self.stack.push(el);
                }
            }

            RawEvent::ElementFoot(_) => self.process_end_tag()?,

            RawEvent::Text(_, text) => self.process_text(text.as_str().to_owned()),
        }

        Ok(())
    }
}
