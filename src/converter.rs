use cmark::*;
use std::io;
use std::io::{ BufWriter, Write };

pub struct Converter {
    indent: usize,
    tightness: bool,
}

/*
 *  Converting methods
 *
 *  A method needs &Event as a parameter iff the node is not a leaf node.
 *  Leaf nodes are HtmlBlock, ThematicBreak, CodeBlock, Text, SoftBreak,
 *  LineBreak, Code and HtmlInline.
 */

impl Converter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            tightness: false,
        }
    }

    pub fn convert<W>(
        &mut self,
        iter: Iter,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        for (node, event) in iter {
            println!("{:?}, {:?}", node, event);

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
            }?
        }

        Ok(())
    }

    fn indent(&self) -> String {
        iter::repeat("    ").take(self.indent).collect::<String>()
    }

    fn convert_blockquote<W>(
        &mut self,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match event {
            &Event::Enter => {
                write!(writer, "{}<blockquote>\n", self.indent())?;
                self.indent += 1;
            },

            &Event::Exit => {
                self.indent -= 1;
                write!(writer, "{}</blockquote>\n", self.indent())?;
            },
        };

        Ok(())
    }

    fn convert_list<W>(
        &mut self,
        ty: &ListType,
        delim: &DelimType,
        start: &StartingNumber,
        tightness: &Tightness,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        self.tightness = (*tightness).into();

        match event {
            &Event::Enter => match ty {
                &ListType::Bullet => write!(writer, "<ul>"),
                &ListType::Ordered => write!(writer, "<ol>"),
            },

            &Event::Exit => match ty {
                &ListType::Bullet => write!(writer, "</ul>"),
                &ListType::Ordered => write!(writer, "</ol>"),
            },
        }?;

        Ok(())
    }

    fn convert_item<W>(
        &self,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match event {
            &Event::Enter => match self.tightness {
                true => write!(writer, "<li>"),
                false => write!(writer, "<li><p>"),
            },

            &Event::Exit => match self.tightness {
                true => write!(writer, "</li>"),
                false => write!(writer, "</p></li>"),
            },
        }?;

        Ok(())
    }

    fn convert_code_block<W>(
        &self,
        info: &InfoString,
        lit: &Literal,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match info.is_empty() {
            true => write!(writer, "<pre><code>{}</code></pre>", lit),
            false => write!(
                writer,
                "<pre><code class=\"{}\">{}</code></pre>",
                info, lit
            ),
        }?;

        Ok(())
    }

    fn convert_html_block<W>(
        &self,
        lit: &Literal,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "{}", lit)?;
        Ok(())
    }

    fn convert_paragraph<W>(
        &self,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match event {
            &Event::Enter => write!(writer, "<p>"),
            &Event::Exit => write!(writer, "</p>"),
        }?;

        Ok(())
    }

    fn convert_heading<W>(
        &self,
        lvl: &HeadingLevel,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match event {
            &Event::Enter => match lvl {
                &HeadingLevel::One => write!(writer, "<h1>"),
                &HeadingLevel::Two => write!(writer, "<h2>"),
                &HeadingLevel::Three => write!(writer, "<h3>"),
                &HeadingLevel::Four => write!(writer, "<h4>"),
                &HeadingLevel::Five => write!(writer, "<h5>"),
                &HeadingLevel::Six => write!(writer, "<h6>"),
            },

            &Event::Exit => match lvl {
                &HeadingLevel::One => write!(writer, "</h1>"),
                &HeadingLevel::Two => write!(writer, "</h2>"),
                &HeadingLevel::Three => write!(writer, "</h3>"),
                &HeadingLevel::Four => write!(writer, "</h4>"),
                &HeadingLevel::Five => write!(writer, "</h5>"),
                &HeadingLevel::Six => write!(writer, "</h6>"),
            },
        }?;

        Ok(())
    }

    fn convert_thematic_break<W>(
        &self,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "<hr />")?;
        Ok(())
    }

    fn convert_text<W>(
        &self,
        lit: &Literal,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "{}", lit)?;
        Ok(())
    }

    fn convert_soft_break<W>(
        &self,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "\n")?;
        Ok(())
    }

    fn convert_line_break<W>(
        &self,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "<br />")?;
        Ok(())
    }

    fn convert_code<W>(
        &self,
        lit: &Literal,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "<code>{}</code>", lit);
        Ok(())
    }

    fn convert_html_inline<W>(
        &self,
        lit: &Literal,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        write!(writer, "{}", lit);
        Ok(())
    }

    fn convert_emph<W>(
        &self,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match event {
            &Event::Enter => write!(writer, "<em>"),
            &Event::Exit => write!(writer, "</em>"),
        }?;

        Ok(())
    }

    fn convert_strong<W>(
        &self,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match event {
            &Event::Enter => write!(writer, "<strong>"),
            &Event::Exit => write!(writer, "</strong>"),
        }?;

        Ok(())
    }

    fn convert_link<W>(
        &self,
        url: &Url,
        title: &Title,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match (event, title.is_empty()) {
            (&Event::Enter, true) =>
                write!(writer, "<a href=\"{}\">", url),

            (&Event::Enter, false) =>
                write!(writer, "<a href=\"{}\" title=\"{}\">", url, title),

            (&Event::Exit, _) =>
                write!(writer, "</a>"),
        }?;

        Ok(())
    }

    fn convert_image<W>(
        &self,
        url: &Url,
        title: &Title,
        event: &Event,
        writer: &mut BufWriter<W>
    ) -> io::Result<()>
        where W: Write
    {
        match (event, title.is_empty()) {
            (&Event::Enter, true) =>
                write!(writer, "<img src=\"{}\" alt=\"", url),

            (&Event::Enter, false) =>
                write!(writer, "<img src=\"{}\" title=\"{}\" alt=\"", url, title),

            (&Event::Exit, _) =>
                write!(writer, "\" />"),
        }?;

        Ok(())
    }
}
