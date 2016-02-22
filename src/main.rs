extern crate pulldown_cmark;
extern crate getopts;

//use getopts::Options;
use std::env;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;
use std::borrow::Cow;
use std::borrow::Cow::{Borrowed, Owned};

use pulldown_cmark::Parser;
use pulldown_cmark::{Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};
use pulldown_cmark::html;
use pulldown_cmark::Event;
use pulldown_cmark::Tag;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("f", "", "set input file name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    //let output = matches.opt_str("o").unwrap();
    let filename = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    let path = Path::new(&filename);
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file
    };
    let mut text = String::new();
    match file.read_to_string(&mut text) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(_) => {},
    }

    //println!("{}", text);

    let mut opts = Options::empty();
    let parser = Parser::new_ext(&text, opts);

    let prelude = "\\documentclass[11pt,a4paper]{report}\n\
                   \\usepackage{listings}\n\
                   \\usepackage{hyperref}\n\
                   \n\
                   \\begin{document}\n";

    let parser = parser.map(|event| match event {
        Event::Text(text) => text.to_string().replace("_", "\\_").replace("&", "\\&"),
        Event::Start(tag) => match tag {
            Tag::Paragraph => "\n".to_string(),
            Tag::Rule => "rule".to_string(),
            Tag::Header(nbr) => "\n\\section{".to_string(),
            Tag::BlockQuote => "blockquote".to_string(),
            Tag::CodeBlock(language) => "\n\\begin{lstlisting}\n".to_string(),
            Tag::List(length) => "\n\\begin{itemize}\n".to_string(),  // TODO: add delim and tight for ast (not needed for html)
            Tag::Item => "\\item ".to_string(),
            Tag::FootnoteDefinition(definition) => "footnote".to_string(),
            Tag::Table(length) => "table".to_string(),
            Tag::TableHead => "tablehead".to_string(),
            Tag::TableRow => "tablerow".to_string(),
            Tag::TableCell => "tablecell".to_string(),
            Tag::Emphasis => "\\textit{".to_string(),
            Tag::Strong => "\\textbf".to_string(),
            Tag::Code => "\\lstinline{".to_string(),
            Tag::Link(caption, link) => "\\url{".to_string() + &*link,
            Tag::Image(caption, image) => image.to_string(),
        },
        Event::End(tag) => match tag {
            Tag::Paragraph => "\n".to_string(),
            Tag::Rule => "rule".to_string(),
            Tag::Header(nbr) => "}\n".to_string(),
            Tag::BlockQuote => "blockquote".to_string(),
            Tag::CodeBlock(language) => "\\end{lstlisting}\n".to_string(),
            Tag::List(length) => "\\end{itemize}\n".to_string(),  // TODO: add delim and tight for ast (not needed for html)
            Tag::Item => "\n".to_string(), // maybe newline?
            Tag::FootnoteDefinition(definition) => "footnote".to_string(),
            Tag::Table(length) => "table".to_string(),
            Tag::TableHead => "tablehead".to_string(),
            Tag::TableRow => "tablerow".to_string(),
            Tag::TableCell => "tablecell".to_string(),
            Tag::Emphasis => "}".to_string(),
            Tag::Strong => "}".to_string(),
            Tag::Code => "}".to_string(),
            Tag::Link(caption, link) => "}".to_string(),
            Tag::Image(caption, image) => image.to_string(),
        },
        Event::Html(html) => "\\url{".to_string() + &*html + "}",
        Event::InlineHtml(html) => "\\url{".to_string() + &*html + "}",
        Event::FootnoteReference(footnote) => footnote.to_string(),
        Event::SoftBreak => "\n".to_string(),
        Event::HardBreak => "\n\\\\".to_string(),
    });

    let output = "output.tex";
    let mut outfile = match File::create(output) {
        Err(why) => panic!("couldn't create {}: {}", output, why),
        Ok(outfile) => outfile,
    };

    match outfile.write_all(prelude.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", output, why)
        },
        Ok(_) => println!("successfully wrote to {}", output),
    }

    for event in parser {
        match outfile.write_all(event.as_bytes()) {
            Err(why) => {
                panic!("couldn't write to {}: {}", output, why)
            },
            Ok(_) => println!("successfully wrote to {}", output),
        }
    }

    match outfile.write_all("\n\\end{document}".as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", output, why)
        },
        Ok(_) => println!("successfully wrote to {}", output),
    }
}
