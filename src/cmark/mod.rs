/*
 *  mostly following:
 *  https://www.mankier.com/3/cmark
 *
 *  good read:
 *  https://stackoverflow.com/a/24148033
 *
 *  I wonder how cmark handles errors
 *
 *  stop designing an API
 *
 *  Here's what the user can do with this module:
 *  1.  Create a parser, feed strings to it, make it parse out a node
 *      representing the entire document.
 *  2.  Get an iterator out of this node.
 *  3.  Iterate through the entire document tree using the iterator. The
 *      iterator should tell you what node you are at and whether you are
 *      entering or exiting the node. (We will have to study the behavior of
 *      events)
 */

use ::bind;

use std::ffi;
use std::os::raw;
use std::str;

/*
 *  Converter from C-style string to Rust string.
 *  Used a bunch in this module.
 */

/// Takes a C string (of C type const char*) and returns a byte-by-byte clone
/// as a Rust String if successful. If `free`, calls C free() on the pointer.
fn raw_to_string(
    raw: *const raw::c_char,
    free: bool
) -> Result<String, str::Utf8Error> {
    let ret;
    match unsafe { ffi::CStr::from_ptr(raw).to_str() } {
        Ok(res) => {
            ret = Ok(res.to_owned());
        },

        Err(e) => {
            ret = Err(e);
        },
    }

    if free {
        unsafe { bind::free(raw as *mut raw::c_void); }
    }

    ret
}

/*
 *  The naive Markdown-to-HTML converter supplied by cmark.
 */

/// Errors that can occur when calling `markdown_to_html`.
#[derive(Debug)]
pub enum MarkdownToHtmlErr {
    Nul(ffi::NulError),
    Utf8(str::Utf8Error),
}

/// Wrapper around `cmark_markdown_to_html`.
pub fn markdown_to_html(
    text: &str,
    len: usize,
    options: Options
) -> Result<String, MarkdownToHtmlErr> {
    let cstring = match ffi::CString::new(text.as_bytes()) {
        Ok(x) => x,
        Err(e) => return Err(MarkdownToHtmlErr::Nul(e)),
    };
    let cstr = cstring.as_c_str();

    let res = unsafe {
        bind::cmark_markdown_to_html(
            cstr.as_ptr(), len, options.as_c_int()
        )
    };

    match raw_to_string(res, true) {
        Ok(x) => Ok(x),
        Err(e) => Err(MarkdownToHtmlErr::Utf8(e)),
    }
}

/*
 *  "External" node that gets exported. Contains all the useful data.
 *  Also relevant enums.
 *
 *  Also, I prefer "block quote" to "blockquote"
 */

/// Wrapper around `cmark_node_type`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Block(Block),
    Inline(Inline),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Document,
    Blockquote,
    List(ListType, DelimType, StartingNumber, Tightness),
    Item,
    CodeBlock(InfoString, Literal), // leaf!
    HtmlBlock(Literal), // leaf!
    CustomBlock,
    Paragraph,
    Heading(HeadingLevel),
    ThematicBreak, // leaf!
}

custom_derive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, NewtypeDisplay)]
    pub struct StartingNumber(isize);
}

custom_derive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, NewtypeDisplay)]
    pub struct Tightness(bool);
}

custom_derive! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, NewtypeDisplay)]
    pub struct InfoString(String);
}

custom_derive! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, NewtypeDisplay)]
    pub struct Literal(String);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeadingLevel {
    One, Two, Three, Four, Five, Six,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Inline {
    Text(Literal), // leaf!
    SoftBreak, // leaf!
    LineBreak, // leaf!
    Code(Literal), // leaf!
    HtmlInline(Literal), // leaf!
    CustomInline,
    Emph,
    Strong,
    Link(Url, Title),
    Image(Url, Title),
}

custom_derive! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, NewtypeDisplay)]
    pub struct Url(String);
}

custom_derive! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, NewtypeDisplay)]
    pub struct Title(String);
}

/// Wrapper around `cmark_list_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListType {
    Bullet,
    Ordered,
}

/// Wrapper around `cmark_delim_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelimType {
    No,
    Period,
    Paren,
}

/*
 *  Iterator that gets exported.
 */

/// Wrapper around `cmark_event_type`.
#[derive(Debug)]
pub enum Event {
    Enter,
    Exit,
}

/// Wrapper around `cmark_iter`.
pub struct Iter {
    raw_root: *mut bind::cmark_node,
    raw_iter: *mut bind::cmark_iter,
    done: bool,
}

impl Iter {
    /// Consumes the parser.
    pub fn from_parser(parser: Parser) -> Self {
        let raw = parser.finish();

        Self {
            raw_root: raw,
            raw_iter: unsafe { bind::cmark_iter_new(raw) },
            done: false,
        }
    }
}

impl Drop for Iter {
    fn drop(&mut self) {
        //  the program stalls if we uncomment this; this probably needs a fix
        //  unsafe { bind::cmark_node_free(self.raw_root); }
        unsafe { bind::cmark_iter_free(self.raw_iter); }
    }
}

impl Iterator for Iter {
    type Item = (Node, Event);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let event = match unsafe {
            bind::cmark_iter_next(self.raw_iter)
        } {
            bind::cmark_event_type::CMARK_EVENT_NONE =>
                panic!("getting CMARK_EVENT_NONE"),
            bind::cmark_event_type::CMARK_EVENT_DONE =>
                {
                    self.done = true;
                    return None;
                },
            bind::cmark_event_type::CMARK_EVENT_ENTER =>
                Event::Enter,
            bind::cmark_event_type::CMARK_EVENT_EXIT =>
                Event::Exit,
        };

        let raw_node = unsafe {
            bind::cmark_iter_get_node(self.raw_iter)
        };

        let node = match unsafe {
            bind::cmark_node_get_type(raw_node)
        } {
            bind::cmark_node_type::CMARK_NODE_NONE =>
                panic!("getting CMARK_NODE_NONE"),

            bind::cmark_node_type::CMARK_NODE_DOCUMENT =>
                Node::Block(Block::Document),
            bind::cmark_node_type::CMARK_NODE_BLOCK_QUOTE =>
                Node::Block(Block::Blockquote),

            bind::cmark_node_type::CMARK_NODE_LIST =>
                Node::Block(Block::List(
                    match unsafe {
                        bind::cmark_node_get_list_type(raw_node)
                    } {
                        bind::cmark_list_type::CMARK_NO_LIST =>
                            panic!("no list no list type"),
                        bind::cmark_list_type::CMARK_BULLET_LIST =>
                            ListType::Bullet,
                        bind::cmark_list_type::CMARK_ORDERED_LIST =>
                            ListType::Ordered,
                    },

                    match unsafe {
                        bind::cmark_node_get_list_delim(raw_node)
                    } {
                        bind::cmark_delim_type::CMARK_NO_DELIM =>
                            DelimType::No,
                        bind::cmark_delim_type::CMARK_PERIOD_DELIM =>
                            DelimType::Period,
                        bind::cmark_delim_type::CMARK_PAREN_DELIM =>
                            DelimType::Paren,
                    },

                    StartingNumber(unsafe {
                        bind::cmark_node_get_list_start(raw_node)
                    } as isize),

                    Tightness(match unsafe {
                        bind::cmark_node_get_list_tight(raw_node)
                    } {
                        0 => false,
                        1 => true,
                        _ => panic!("not 0 or 1"),
                    })
                )),

            bind::cmark_node_type::CMARK_NODE_ITEM =>
                Node::Block(Block::Item),

            bind::cmark_node_type::CMARK_NODE_CODE_BLOCK =>
                Node::Block(Block::CodeBlock(
                    InfoString::from(raw_to_string(unsafe {
                        bind::cmark_node_get_fence_info(raw_node)
                    }, false).expect("bad info string")),

                    Literal::from(raw_to_string(unsafe {
                        bind::cmark_node_get_literal(raw_node)
                    }, false).expect("bad literal")),
                )),

            bind::cmark_node_type::CMARK_NODE_HTML_BLOCK =>
                Node::Block(Block::HtmlBlock(
                    Literal::from(raw_to_string(unsafe {
                        bind::cmark_node_get_literal(raw_node)
                    }, false).expect("bad literal"))
                )),

            bind::cmark_node_type::CMARK_NODE_CUSTOM_BLOCK =>
                Node::Block(Block::CustomBlock),
            bind::cmark_node_type::CMARK_NODE_PARAGRAPH =>
                Node::Block(Block::Paragraph),

            bind::cmark_node_type::CMARK_NODE_HEADING =>
                Node::Block(Block::Heading(match unsafe {
                    bind::cmark_node_get_heading_level(raw_node)
                } {
                    1 => HeadingLevel::One,
                    2 => HeadingLevel::Two,
                    3 => HeadingLevel::Three,
                    4 => HeadingLevel::Four,
                    5 => HeadingLevel::Five,
                    6 => HeadingLevel::Six,
                    _ => panic!("not a valid heading level"),
                })),

            bind::cmark_node_type::CMARK_NODE_THEMATIC_BREAK =>
                Node::Block(Block::ThematicBreak),

            bind::cmark_node_type::CMARK_NODE_TEXT =>
                Node::Inline(Inline::Text(
                    Literal::from(raw_to_string(unsafe {
                        bind::cmark_node_get_literal(raw_node)
                    }, false).expect("bad literal"))
                )),

            bind::cmark_node_type::CMARK_NODE_SOFTBREAK =>
                Node::Inline(Inline::SoftBreak),
            bind::cmark_node_type::CMARK_NODE_LINEBREAK =>
                Node::Inline(Inline::LineBreak),

            bind::cmark_node_type::CMARK_NODE_CODE =>
                Node::Inline(Inline::Code(
                    Literal::from(raw_to_string(unsafe {
                        bind::cmark_node_get_literal(raw_node)
                    }, false).expect("bad literal"))
                )),

            bind::cmark_node_type::CMARK_NODE_HTML_INLINE =>
                Node::Inline(Inline::HtmlInline(
                    Literal::from(raw_to_string(unsafe {
                        bind::cmark_node_get_literal(raw_node)
                    }, false).expect("bad literal"))
                )),

            bind::cmark_node_type::CMARK_NODE_CUSTOM_INLINE =>
                Node::Inline(Inline::CustomInline),
            bind::cmark_node_type::CMARK_NODE_EMPH =>
                Node::Inline(Inline::Emph),
            bind::cmark_node_type::CMARK_NODE_STRONG =>
                Node::Inline(Inline::Strong),

            bind::cmark_node_type::CMARK_NODE_LINK =>
                Node::Inline(Inline::Link(
                    Url::from(raw_to_string(unsafe {
                        bind::cmark_node_get_url(raw_node)
                    }, false).expect("bad url")),

                    Title::from(raw_to_string(unsafe {
                        bind::cmark_node_get_title(raw_node)
                    }, false).expect("bad title"))
                )),

            bind::cmark_node_type::CMARK_NODE_IMAGE =>
                Node::Inline(Inline::Image(
                    Url::from(raw_to_string(unsafe {
                        bind::cmark_node_get_url(raw_node)
                    }, false).expect("bad url")),

                    Title::from(raw_to_string(unsafe {
                        bind::cmark_node_get_title(raw_node)
                    }, false).expect("bad title"))
                )),
        };

        //  we are done if we get an Exit to the Document
        if let (&Node::Block(Block::Document), &Event::Exit) = (&node, &event) {
            self.done = true;
        }

        Some((node, event))
    }
}

/*
 *  Parser that gets exported.
 */

/// Errors that can occur when calling any of the parsing functions.
#[derive(Debug)]
pub enum ParserErr {
    Nul(ffi::NulError),
}

/*  doesn't work now
/// Wrapper around `cmark_parse_document`.
fn parse_document(
    buffer: &str,
    len: usize,
    options: Options
) -> Result<Node, ParserErr> {
    match ffi::CString::new(buffer) {
        Ok(cstring) => Ok(Node::from_raw_undropped(
            unsafe { bind::cmark_parse_document(
                cstring.as_ptr(),
                len,
                options.as_c_int()
            ) }
        )),

        Err(x) => Err(ParserErr::Nul(x)),
    }
}*/

/// Wrapper around `cmark_parser`.
pub struct Parser {
    raw: *mut bind::cmark_parser,
}

impl Parser {
    pub fn new(options: Options) -> Parser {
        Parser {
            raw: unsafe { bind::cmark_parser_new(options.as_c_int()) },
        }
    }

    pub fn feed(&mut self, buffer: &str, len: usize) -> Result<(), ParserErr> {
        match ffi::CString::new(buffer) {
            Ok(cstring) => unsafe {
                bind::cmark_parser_feed(self.raw, cstring.as_ptr(), len);
                Ok(())
            },

            Err(x) => Err(ParserErr::Nul(x)),
        }
    }

    //  finish should probably consume the Parser
    fn finish(self) -> *mut bind::cmark_node {
        unsafe { bind::cmark_parser_finish(self.raw) }
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe { bind::cmark_parser_free(self.raw); }
    }
}

/*
 *  Rendering functions that get exported.
 */

/// Errors that can occur when calling any of the rendering functions.
#[derive(Debug)]
enum RenderErr {
    Utf8(str::Utf8Error),
}

fn render_html(
    raw_root: *mut bind::cmark_node,
    options: Options
) -> Result<String, RenderErr> {
    let res = unsafe {
        bind::cmark_render_html(
            raw_root, options.bits as raw::c_int
        )
    };

    match raw_to_string(res, true) {
        Ok(x) => Ok(x),
        Err(e) => Err(RenderErr::Utf8(e)),
    }
}

/*
 *  Options affecting rendering and parsing.
 */

/// Represents the options `CMARK_OPT_...`.
///
/// The correspondences between the Rust constants and the `cmark` options are
/// listed below.
///
/// *   `Options::DEFAULT` to `CMARK_OPT_DEFAULT`
/// *   `Options::SOURCE_POS` to `CMARK_OPT_SOURCEPOS`
/// *   `Options::HARD_BREAKS` to `CMARK_OPT_HARDBREAKS`
/// *   `Options::SAFE` to `CMARK_OPT_SAFE`
/// *   `Options::NO_BREAKS` to `CMARK_OPT_NOBREAKS`
/// *   `Options::NORMALIZE` to `CMARK_OPT_NORMALIZE`
/// *   `Options::VALIDATE_UTF8` to `CMARK_OPT_VALIDATE_UTF8`
/// *   `Options::SMART` to `CMARK_OPT_SMART`
bitflags! {
    pub struct Options: raw::c_uint {
        //  because Default is already taken...
        const DEFAULT       = 0b0000_0000_0000;
        const SOURCE_POS    = 0b0000_0000_0010;
        const HARD_BREAKS   = 0b0000_0000_0100;
        const SAFE          = 0b0000_0000_1000;
        const NO_BREAKS     = 0b0000_0001_0000;
        const NORMALIZE     = 0b0001_0000_0000;
        const VALIDATE_UTF8 = 0b0010_0000_0000;
        const SMART         = 0b0100_0000_0000;
    }
}

/// Less typing
impl Options {
    fn as_c_int(&self) -> raw::c_int {
        self.bits as raw::c_int
    }
}
