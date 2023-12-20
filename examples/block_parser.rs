use parsely::{any, char_if, result_ext::*, token, until, Lex, Parse, ParseResult};

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

fn block_parser(input: &str) -> ParseResult<Vec<Node>> {
    node.many(1..).then_end().parse(input).offset(input)
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
        .parse(input)
        .offset(input)?;

    if input == remaining {
        return Err(parsely::Error::no_match(input));
    } else {
        Ok((output, remaining))
    }
}

fn open_tag(input: &str) -> ParseResult<'_, String> {
    " ".optional()
        .skip_then(
            char_if(|c| !c.is_ascii_whitespace())
                .many(1..)
                .map(str::to_string),
        )
        .then_skip(" ".optional())
        .pad_with("{@", "@}")
        .parse(input)
        .offset(input)
}

fn close_tag(tag: &str) -> impl Lex + '_ {
    " ".optional()
        .then("end ")
        .then(token(tag).many(1..))
        .then(" ".optional())
        .pad_with("{@", "@}")
}

fn block(input: &str) -> ParseResult<'_, Block> {
    let (tag, remaining) = open_tag(input).offset(input)?;

    let (nodes, remaining) = node
        .many(..)
        .then_skip(close_tag(tag.as_str()))
        .parse(remaining)
        .offset(input)?;

    let block = Block { tag, nodes };
    Ok((block, remaining))
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
fn test_node_matches_leading_content() {
    let (matched, remaining) =
        node("pre foo>{@ foo @}these{@ foo @}are{@ foo @}nested{@ end foo @}{@ end foo @}")
            .unwrap();

    assert_eq!(matched, Node::from_content("pre foo>"));
}

fn main() {
    // successful
    println!("{:#?}", block_parser("content before blocks {@ empty @}{@ end empty @} content between blocks {@ empty @}{@ end empty @} content after blocks"));
    println!("{:#?}", block_parser("{@ foo @}inside foo{@ end foo @}"));
    println!("{:#?}", block_parser("{@ foo @}this is the first block's content{@ foo @}this is a nested block inside the first{@ end foo @}{@ end foo @}"));
    println!("{:#?}", block_parser("content before foo block {@ foo @}this is the first block's content{@ foo @}this is a nested block inside the first{@ end foo @}this is more of the first block's content{@ end foo @}"));
    println!("{:#?}", block_parser("content before foo block {@ foo @}content before bar block {@ bar @}this is a nested bar block inside the foo block{@ end bar @} content after bar block{@ end foo @} content after foo block"));

    // errors
    println!("{:#?}", block_parser("content before foo block {@ foo @}content before bar block {@ bar @}this is a nested bar block inside the foo block{@ end bar @} content after bar block{@ end bar @} content after foo block"));
    println!("{:#?}", block_parser("{@ foo @}{@ end bar @}"));
    println!("{:#?}", content("{@ foo @}{@ end bar @}"));
}
