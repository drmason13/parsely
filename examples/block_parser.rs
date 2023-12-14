use parsely::{any, char_if, result_ext::*, until, Lex, Parse, ParseResult};

#[derive(PartialEq, Debug)]
pub enum Node {
    Content(String),
    Block(Block),
}

impl Node {
    pub fn from_content(content: &str) -> Self {
        if content.is_empty() {
            // Node::Empty
            Node::Content(String::from(content))
        } else {
            Node::Content(content.to_string())
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Block {
    tag: String,
    nodes: Vec<Node>,
}

#[derive(PartialEq, Debug)]
pub enum Tag {
    Open(String),
    Close(String),
}

impl Tag {
    pub fn close(name: &str) -> Tag {
        Tag::Close(name.to_string())
    }

    pub fn open(name: &str) -> Tag {
        Tag::Open(name.to_string())
    }
}

fn full_node(input: &str) -> ParseResult<Node> {
    node.then_end().parse(input).offset(input)
}

fn node(input: &str) -> ParseResult<Node> {
    block
        .map(Node::Block)
        .or(content)
        .parse(input)
        .offset(input)
}

fn content(input: &str) -> ParseResult<Node> {
    let (output, remaining) = until("{@")
        .map(Node::from_content)
        .or(any().many(1..).map(Node::from_content).then_end())
        .parse(dbg!(input))
        .offset(input)?;

    if input == remaining {
        return Err(parsely::Error::no_match(input));
    } else {
        Ok((output, remaining))
    }
}

fn open_tag() -> impl Parse<Output = Tag> {
    " ".optional()
        .skip_then(
            char_if(|c| !c.is_ascii_whitespace())
                .many(1..)
                .map(Tag::open),
        )
        .then_skip(" ".optional())
        .pad_with("{@", "@}")
}

fn close_tag() -> impl Parse<Output = Tag> {
    " ".optional()
        .then("end ")
        .skip_then(
            char_if(|c| !c.is_ascii_whitespace())
                .many(1..)
                .map(Tag::close),
        )
        .then_skip(" ".optional())
        .pad_with("{@", "@}")
}

fn block_inner(input: &str) -> ParseResult<Vec<Node>> {
    let mut output = Vec::new();
    let mut working_input = input;

    let mut working_remaining = input;

    while let Ok((matched, remaining)) = node.parse(working_input).offset(input) {
        output.push(matched);
        if remaining == working_input {
            // no more progress can be made!
            break;
        } else {
            working_remaining = remaining;
            working_input = remaining;
        }
    }

    Ok((output, working_remaining))
}

fn block(input: &str) -> ParseResult<'_, Block> {
    let (((open_tag, nodes), close_tag), remaining) = open_tag()
        .then(block_inner)
        .then(close_tag())
        .parse(input)
        .offset(input)?;

    if let Tag::Open(tag) = open_tag {
        let Tag::Close(closer) = close_tag else {
            unreachable!()
        };
        if tag != closer {
            println!("tags don't match!");
            return Err(parsely::Error::no_match(input));
        }

        let block = Block { tag, nodes };
        Ok((block, remaining))
    } else {
        unreachable!()
    }
}

#[test]
fn test_no_content() {
    let err = node("").unwrap_err();

    assert_eq!(err.matched(), "");
}

#[test]
fn test_no_blocks() {
    let (block, _) = node("no blocks").unwrap();

    assert_eq!(block, Node::Content(String::from("no blocks")),)
}

#[test]
fn test_nested() {
    let (node, _) =
        node("{@ foo @} this is the first block's content {@ foo @}this is a nested block inside the first{@ end foo @}{@ end foo @}").unwrap();

    assert_eq!(
        node,
        Node::Block(Block {
            tag: String::from("foo"),
            nodes: vec![
                Node::Content(String::from(" this is the first block's content ")),
                Node::Block(Block {
                    tag: String::from("foo"),
                    nodes: vec![Node::Content(String::from(
                        "this is a nested block inside the first"
                    )),]
                }),
            ]
        })
    )
}

#[test]
fn test_full_node() {
    let (matched, remaining) =
        node("pre foo>{@ foo @}these{@ foo @}are{@ foo @}nested{@ end foo @}{@ end foo @}")
            .unwrap();

    assert_eq!(matched, Node::from_content("pre foo>"));
}

fn main() {
    println!(
        "{:#?}",
        node("content before block {@ empty @}{@ end empty @} content after block")
    );
    println!("{:#?}", node("{@ foo @}inside foo{@ end foo @}"));
}
