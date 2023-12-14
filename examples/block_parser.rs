use parsely::{result_ext::*, until, Lex, Parse, ParseResult};

#[derive(PartialEq, Debug)]
pub enum Node {
    Empty,
    Content(String),
    Block { tag: Tag, nodes: Vec<Node> },
}

#[derive(PartialEq, Debug)]
pub enum Tag {
    Foo,
    Bar,
}

impl Node {
    pub fn from_content(content: &str) -> Node {
        if content.is_empty() {
            Node::Empty
        } else {
            Node::Content(content.to_string())
        }
    }
}

fn node(input: &str) -> ParseResult<Node> {
    match until("{@").lex(input).offset(input) {
        Ok((content, remaining)) => {
            // we found some content followed by a block!
            let ((tag, nodes), remaining) = block.parse(input).offset(input)?;
            Ok((Node::Block { tag, nodes }, remaining))
        }
        // no opening blocks in the entire input, so it is just content
        Err(_) => Ok((Node::from_content(input), "")),
    }
}

fn tag(name: &'static str) -> impl Lex {
    " ".optional()
        .then(name)
        .then(" ".optional())
        .pad_with("{@", "@}")
}

fn block(input: &str) -> ParseResult<'_, (Tag, Vec<Node>)> {
    node.many(..)
        .pad_with(tag("foo"), tag("end foo"))
        .map(|v| (Tag::Foo, v))
        .parse(input)
        .offset(input)
}

#[test]
fn test_no_content() {
    let (node, _) = node("").unwrap();

    assert_eq!(node, Node::Empty,)
}

#[test]
fn test_no_blocks() {
    let (block, _) = node("no blocks").unwrap();

    assert_eq!(block, Node::Content(String::from("no blocks")),)
}

#[test]
fn test_nested() {
    let (node, _) =
        node("{@ foo @}this is the first block's content{@ foo @}this is a nested block inside the first{@ end foo @}{@ end foo @}").unwrap();

    assert_eq!(
        node,
        Node::Block(vec![
            Node::Content(String::from(" this is the first block's content ")),
            Node::Block(vec![Node::Content(String::from(
                " this is the nested block "
            )),]),
            Node::Content(String::from(" ")),
        ])
    )
}

fn main() {
    println!("{:#?}", node("{@ foo @}inside foo{@ end foo @}"));
}
