use converters::Converter;

use cmark::*;
use asset::*;

use std::io;
use std::io::{ Read, Write };

use std::iter;

use std::marker::PhantomData;

lazy_static! {
    static ref INDENT: String = String::from("    "); 

    static ref HEADER_PRE_ASSETS: String = format!(
        "<!DOCTYPE html>\n\
<html>\n\
{0}<head>\n\
{0}{0}<meta charset=\"UTF-8\">\n\
{0}{0}<title>Title</title>\n",
        &*INDENT
    );

    static ref HEADER_POST_ASSETS: String = format!(
        "{0}</head>\n\
\n\
{0}<body>\n\
{0}{0}<div class=\"container u-full-width\">\n",
        &*INDENT
    );

    static ref FOOTER: String = format!(
        "{0}{0}</div>\n\
{0}</body>\n\
{0}<script>hljs.initHighlightingOnLoad();</script>\n\
</html>",
        &*INDENT
    );
}

pub struct BasicConverter<'a> {
    indent: usize,
    tightness: bool,
    phantom: PhantomData<&'a ()>,
}

pub struct BasicData<'a> {
    assets: &'a Vec<Asset>,
    dist: usize,
}

impl<'a> BasicData<'a> {
    pub fn new(assets: &'a Vec<Asset>, dist: usize) -> Self {
        Self {
            assets: assets,
            dist: dist,
        }
    }
}

/*
 *  Converting methods
 *
 *  A method needs &Event as a parameter iff the node is not a leaf node.
 *  Leaf nodes are HtmlBlock, ThematicBreak, CodeBlock, Text, SoftBreak,
 *  LineBreak, Code and HtmlInline.
 */

impl<'a> BasicConverter<'a> {

    fn write_header<W: Write>(
        &mut self,
        writer: &mut W,
        assets: &Vec<Asset>,
        dist: usize
    ) -> io::Result<()> {
        write!(writer, "{}", &*HEADER_PRE_ASSETS)?;

        self.write_assets(writer, assets, dist)?;

        write!(writer, "{}", &*HEADER_POST_ASSETS)?;

        self.indent = 2;

        Ok(())
    }

    fn write_assets<W: Write>(
        &mut self,
        writer: &mut W,
        assets: &Vec<Asset>,
        dist: usize
    ) -> io::Result<()> {
        for asset in assets {
            match asset.asset_type() {
                &AssetType::Css => {
                    write!(
                        writer,
                        "<link rel=\"stylesheet\" href=\"{}{}\" type=\"text/css\">\n",
                        "../".repeat(dist),
                        asset.path().display()
                    )
                },

                &AssetType::Js => {
                    write!(
                        writer,
                        "<script src=\"{}{}\" type=\"text/javascript\"></script>\n",
                        "../".repeat(dist),
                        asset.path().display()
                    )
                },

                &AssetType::Other => Ok(()),
            }?;
        }

        Ok(())
    }

    fn write_footer<W: Write>(
        &mut self,
        writer: &mut W,
    ) -> io::Result<()> {
        write!(writer, "{}", &*FOOTER)?;

        Ok(())
    }

    fn repeat_indent(n: usize) -> String {
        iter::repeat((*INDENT).clone()).take(n).collect::<String>()
    }

    fn make_indent(&self) -> String {
        Self::repeat_indent(self.indent)
    }

    fn convert_blockquote<W: Write>(
        &mut self,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => {
                write!(writer, "{}<blockquote>\n", self.make_indent())?;
                self.indent += 1;
            },

            &Event::Exit => {
                self.indent -= 1;
                write!(writer, "{}</blockquote>\n", self.make_indent())?;
            },
        };

        Ok(())
    }

    fn convert_list<W: Write>(
        &mut self,
        ty: &ListType,
        delim: &DelimType,
        start: &StartingNumber,
        tightness: &Tightness,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        self.tightness = (*tightness).into();

        match event {
            &Event::Enter => {
                match ty {
                    &ListType::Bullet => write!(writer, "{}<ul>\n", self.make_indent()),
                    &ListType::Ordered => write!(writer, "{}<ol>\n", self.make_indent()),
                }?;

                self.indent += 1;
            },

            &Event::Exit => {
                self.indent -= 1;

                match ty {
                    &ListType::Bullet => write!(writer, "{}</ul>\n", self.make_indent()),
                    &ListType::Ordered => write!(writer, "{}</ol>\n", self.make_indent()),
                }?;
            },
        };

        Ok(())
    }

    fn convert_item<W: Write>(
        &mut self,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => {
                match self.tightness {
                    true => write!(writer, "{}<li>\n", self.make_indent()),
                    false => write!(writer, "{}<li><p>\n", self.make_indent()),
                }?;

                self.indent += 1;
            },

            &Event::Exit => {
                self.indent -= 1;
                
                match self.tightness {
                    true => write!(writer, "{}</li>\n", self.make_indent()),
                    false => write!(writer, "{}</p></li>\n", self.make_indent()),
                }?;
            },
        };

        Ok(())
    }

    fn convert_code_block<W: Write>(
        &self,
        info: &InfoString,
        lit: &Literal,
        writer: &mut W
    ) -> io::Result<()> {
        match info.is_empty() {
            true => write!(
                writer,
                "{}<pre><code>\n{}\n</code></pre>\n",
                self.make_indent(), lit
            ),

            false => write!(
                writer,
                "{}<pre><code class=\"{}\">\n{}\n</code></pre>\n",
                self.make_indent(), info, lit
            ),
        }?;

        Ok(())
    }

    fn convert_html_block<W: Write>(
        &self,
        lit: &Literal,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "{}{}\n", self.make_indent(), lit)?;
        Ok(())
    }

    fn convert_paragraph<W: Write>(
        &mut self,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => {
                write!(writer, "{}<p>", self.make_indent())?;
                self.indent += 1;
            },

            &Event::Exit => {
                self.indent -= 1;
                write!(writer, "</p>\n")?;
            },
        };

        Ok(())
    }

    fn convert_heading<W: Write>(
        &mut self,
        lvl: &HeadingLevel,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => {
                match lvl {
                    &HeadingLevel::One => write!(writer, "{}<h1>", self.make_indent()),
                    &HeadingLevel::Two => write!(writer, "{}<h2>", self.make_indent()),
                    &HeadingLevel::Three => write!(writer, "{}<h3>", self.make_indent()),
                    &HeadingLevel::Four => write!(writer, "{}<h4>", self.make_indent()),
                    &HeadingLevel::Five => write!(writer, "{}<h5>", self.make_indent()),
                    &HeadingLevel::Six => write!(writer, "{}<h6>", self.make_indent()),
                }?;

                self.indent += 1;
            },

            &Event::Exit => {
                self.indent -= 1;
                
                match lvl {
                    &HeadingLevel::One => write!(writer, "</h1>\n"),
                    &HeadingLevel::Two => write!(writer, "</h2>\n"),
                    &HeadingLevel::Three => write!(writer, "</h3>\n"),
                    &HeadingLevel::Four => write!(writer, "</h4>\n"),
                    &HeadingLevel::Five => write!(writer, "</h5>\n"),
                    &HeadingLevel::Six => write!(writer, "</h6>\n"),
                }?;
            },
        };

        Ok(())
    }

    fn convert_thematic_break<W: Write>(
        &self,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "{}<hr />\n", self.make_indent())?;
        Ok(())
    }

    fn convert_text<W: Write>(
        &self,
        lit: &Literal,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "{}", lit)?;
        Ok(())
    }

    fn convert_soft_break<W: Write>(
        &self,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "\n")?;
        Ok(())
    }

    fn convert_line_break<W: Write>(
        &self,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "<br />\n")?;
        Ok(())
    }

    fn convert_code<W: Write>(
        &self,
        lit: &Literal,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "<code>{}</code>", lit);
        Ok(())
    }

    fn convert_html_inline<W: Write>(
        &self,
        lit: &Literal,
        writer: &mut W
    ) -> io::Result<()> {
        write!(writer, "{}", lit);
        Ok(())
    }

    fn convert_emph<W: Write>(
        &self,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => write!(writer, "<em>"),
            &Event::Exit => write!(writer, "</em>"),
        }?;

        Ok(())
    }

    fn convert_strong<W: Write>(
        &self,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => write!(writer, "<strong>"),
            &Event::Exit => write!(writer, "</strong>"),
        }?;

        Ok(())
    }

    fn convert_link<W: Write>(
        &self,
        url: &Url,
        title: &Title,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => match title.is_empty() {
                true => write!(writer, "<a href=\"{}\">", url),
                false => write!(writer, "<a href=\"{}\" title=\"{}\">", url, title),
            },

            &Event::Exit => write!(writer, "</a>"),
        }?;

        Ok(())
    }

    fn convert_image<W: Write>(
        &self,
        url: &Url,
        title: &Title,
        event: &Event,
        writer: &mut W
    ) -> io::Result<()> {
        match event {
            &Event::Enter => match title.is_empty() {
                true => write!(writer, "<img src=\"{}\" alt=\"", url),
                false => write!(writer, "<img src=\"{}\" title=\"{}\" alt=\"", url, title),
            },

            &Event::Exit => write!(writer, "\" />"),
        }?;

        Ok(())
    }

}

impl<'a> Converter for BasicConverter<'a> {

    type MoreData = BasicData<'a>;
    
    fn new() -> Self {
        Self {
            indent: 0,
            tightness: false,
            phantom: PhantomData,
        }
    }

    fn convert<R: Read, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        data: Self::MoreData,
    ) -> io::Result<()> {
        let assets = data.assets;
        let dist = data.dist;

        let mut read_buffer = String::new();
        reader.read_to_string(&mut read_buffer).unwrap();

        let iter = Iter::from_parser({
            let mut parser = Parser::new(Options::DEFAULT);
            parser.feed(read_buffer.as_str(), read_buffer.len()).expect(
                "feeding failed"
            );
            parser
        });

        self.write_header(writer, assets, dist)?;

        for (node, event) in iter {
            match node {
                Node::Block(Block::Document) => Ok(()),

                Node::Block(Block::Blockquote) =>
                    self.convert_blockquote(&event, writer),

                Node::Block(Block::List(ty, delim, start, tightness)) =>
                    self.convert_list(
                        &ty,
                        &delim,
                        &start,
                        &tightness,
                        &event,
                        writer
                    ),

                Node::Block(Block::Item) =>
                    self.convert_item(&event, writer),

                Node::Block(Block::CodeBlock(info, lit)) =>
                    self.convert_code_block(&info, &lit, writer),

                Node::Block(Block::HtmlBlock(lit)) =>
                    self.convert_html_block(&lit, writer),

                Node::Block(Block::CustomBlock) => {
                    println!("custom blocks not implemented yet");
                    Ok(())
                },

                Node::Block(Block::Paragraph) =>
                    self.convert_paragraph(&event, writer),

                Node::Block(Block::Heading(lvl)) =>
                    self.convert_heading(&lvl, &event, writer),

                Node::Block(Block::ThematicBreak) =>
                    self.convert_thematic_break(writer),

                Node::Inline(Inline::Text(lit)) =>
                    self.convert_text(&lit, writer),

                Node::Inline(Inline::SoftBreak) =>
                    self.convert_soft_break(writer),

                Node::Inline(Inline::LineBreak) =>
                    self.convert_line_break(writer),

                Node::Inline(Inline::Code(lit)) =>
                    self.convert_code(&lit, writer),

                Node::Inline(Inline::HtmlInline(lit)) =>
                    self.convert_html_inline(&lit, writer),

                Node::Inline(Inline::CustomInline) => {
                    println!("custom inlines not implemented yet");
                    Ok(())
                },

                Node::Inline(Inline::Emph) =>
                    self.convert_emph(&event, writer),

                Node::Inline(Inline::Strong) =>
                    self.convert_strong(&event, writer),

                Node::Inline(Inline::Link(url, title)) =>
                    self.convert_link(&url, &title, &event, writer),

                Node::Inline(Inline::Image(url, title)) =>
                    self.convert_image(&url, &title, &event, writer),
            }?;
        }

        self.write_footer(writer)?;

        Ok(())
    }

}
