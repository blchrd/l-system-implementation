# L-System implementation

This repo contains a simple implementation in Rust of [L-System](https://en.m.wikipedia.org/wiki/L-system) to generate images.

## Usage

```
cargo run -- [-f filepath] [-o file]
```

The rendering part can be a little long if you want a high number of iteration. You can use the --release flag if you want to speed up the rendering:

```
cargo run --release -- [-f filepath] [-o file]
```

You have sample file in the `sample` folder.

By default, the rendered image will be in the `render` folder, you can customize the output file with the `-o` parameter.

Next and last step for this little project, is to randomize L-System if no input file in defined.