pub mod format;
pub mod parser;

pub use format::{Document, Hunk, HunkPos};

#[cfg(test)]
mod tests {
    use crate::format::Document;

    static TEST_DATA: &'static str = r#"This is a markdown file
=======================

... and it's also going to contain test fixture data.

The markdown is full of idioms and unidiomatic things alike.

The contents don't really matter, mostly.
They're freetext, and you can use markdown to describe whatever you want.

Except these:

[testmark]:# (this-is-the-data-name)
```text
the content of this code block is data which can be read,
and *replaced*, by testmark.
```

That's not a regular code block.
I mean, it is -- but be sure to look at this file in a "raw" mode.
There's also a comment above it, which tells testmark to look at it:

```
[testmark]:# (this-is-the-data-name)
```

That comment, coming right before a triple-backtick code block, is a signal.
It tells testmark to look at the codeblock, and also gives that code block a name.


Multiple data hunks per file
----------------------------

You can have more than one block like that in a file:

[testmark]:# (more-data)
```go
func OtherMarkdownParsers() (shouldHighlight bool) {
	return true
}
```

### the headings don't matter

Every other markdown feature, like the headings, are totally ignored.
That structure is for *you*, human, as you write documentation together with your data.


Editing
-------

Testmark can edit a file like this one, and replace the code block contents according to the name given in the comment.
(It's kind of like a big map of strings, in that regard.)

Usually, a human writes the testmark file.
If the human wants to programmatically populate things, the human writes out the code block and names it with the magic comment format,
and then runs some tool that updates the content.

Some libraries may also be able to create a testmark file purely programmatically, but this is usually more complicated,
and makes it harder to control the rest of the document...
which, presumably, you do still want to fill with prose (and markdown-formatted) descriptions of the data.


One note
--------

There is one thing this format is bad at:
you can't easily describe data that doesn't have a trailing linebreak.

[testmark]:# (cannot-describe-no-linebreak)
```
A markdown codeblock always has a trailing linebreak before its close indicator, you see.
```

That's a problem in many formats though, frankly.
"#;

    #[test]
    fn parse_example() {
        let doc = Document::from_string(&TEST_DATA);
        assert_eq!(doc.hunks().len(), 3);

        let hunk = &doc.hunks()[0];
        assert_eq!(hunk.name(), "this-is-the-data-name");
        assert_eq!(hunk.info(), Some(&"text".to_string()));

        let hunk = &doc.hunks()[1];
        assert_eq!(hunk.name(), "more-data");
        assert_eq!(hunk.info(), Some(&"go".to_string()));

        let hunk = &doc.hunks()[2];
        assert_eq!(hunk.name(), "cannot-describe-no-linebreak");
        assert_eq!(hunk.info(), None);
    }

    static MUTATION_ORIG: &'static str = r#"
# Header text

[testmark]:# (mutation-test)
```text
This is the original
```

Footer text
"#;
    static MUTATION_NEW: &'static str = r#"
# Header text

[testmark]:# (mutation-test-2)
```rust
pub fn main() {
  println!("Hello World!");
}
```

Footer text
"#;
    #[test]
    fn test_mutation() {
        let mut doc = Document::from_string(MUTATION_ORIG);
        let hunk = &mut doc.hunks_mut()[0];
        hunk.set_name("mutation-test-2");
        hunk.set_info(Some("rust".to_string()));
        hunk.set_data(
            r#"pub fn main() {
  println!("Hello World!");
}"#
            .as_bytes()
            .to_vec(),
        );
        let new_doc = doc.render();
        assert_eq!(&new_doc, MUTATION_NEW.as_bytes());
    }
}
