# writer2

A minimal static site generator that converts Markdown to HTML, with some
special syntax for typesetting contents in topics such as linguistics,
mathematics and programming.

## Building and running

Simply build and run the executable without specifying any arguments:

```
cargo run
```

A help message will be displayed, which should be helpful.

## Project Documentation

To build the documentation for this project, run:

```
cargo rustdoc --open -- --document-private-items
```

This should be of interest primarily to the maintainers of this project, of
which there is currently only one.

## License

See [here](LICENSE.md).
