# Pixaki Converter

I love the app [Pixaki](https://pixaki.com/), but I recently moved off of using an iPad for my creative endeavors. This CLI tool is an attempt at making my decade+ of drawings available in other apps, like [Aseprite](https://www.aseprite.org/).

Yes, this is "vibe coded". I hope to make improvements as I work through the conversion process with my backup.

## Usage

To convert a Pixaki file to Aseprite, use the following command:

```bash
cargo run --bin pixelartconvert -- <path_to_pixaki_file> <path_to_output_aseprite_file>
```

For example:

```bash
cargo run --bin pixelartconvert -- tests/data/fox_smile.pixaki output.aseprite
```

## Building

To build the entire workspace:

```bash
cargo build
```

To build just the CLI:

```bash
cargo build -p pixelartconvert
```

## Running Tests

To run all tests across all crates:

```bash
cargo test
```

## License

Apache 2.0
